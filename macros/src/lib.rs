use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Ident};

#[proc_macro_derive(Table, attributes(table_name))]
pub fn table_derive(token: TokenStream) -> TokenStream {
    let ast = syn::parse(token).unwrap();
    impl_table_trait(&ast)
}

fn impl_table_trait(ast: &syn::DeriveInput) -> TokenStream {
    let t = &ast.ident;

    let table_name = ast
        .attrs
        .iter()
        .find(|x| x.path().is_ident("table_name"))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().map(|x| x.value()).ok())
        .unwrap_or(t.to_string().to_lowercase());

    let fields = match &ast.data {
        Data::Struct(e) => &e.fields,
        _ => panic!("Derive only pemit in structs"),
    }
    .iter()
    .filter_map(|x| x.ident.as_ref())
    .collect::<Vec<&Ident>>();

    quote! {
        impl crate::database::repository::Table for #t {
            fn name() -> String {
                #table_name.to_string()
            }

            fn get_fields(self) -> Vec<crate::database::repository::TypeTable> {
                vec![
                    #(self.#fields.into()),*
                ]
            }

            fn columns() -> Vec<&'static str> {
                vec![
                    #(stringify!(#fields)),*
                ]
            }
        }
    }
    .into()
}

#[proc_macro_derive(
    FromPgRow,
    attributes(FromStr, default, offset_timestamp, hours, minutes, seconds)
)]
pub fn from_pg_row(token: TokenStream) -> TokenStream {
    let tmp = syn::parse(token).unwrap();

    impl_from_pg_row(&tmp)
}

fn impl_from_pg_row(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(e) => &e.fields,
        _ => panic!("The type {} isn't a struct", stringify!(name)),
    }.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let attrs = &field.attrs;

        if attrs.iter().any(|x| x.path().is_ident("FromStr")) {
            if attrs.iter().any(|x|x.path().is_ident("default")) {
                quote! { #field_name: sqlx::Row::get::<'_, &str, _>(&value, stringify!(#field_name)).parse().unwrap_or_default() }
            } else {
                quote! { #field_name: sqlx::Row::get::<'_, &str, _>(&value, stringify!(#field_name)).parse().unwrap() }
            }
        } else if let Some(e) = attrs.iter().find(|x| x.path().is_ident("offset_timestamp")) {

            let exp = e.parse_args::<syn::ExprTuple>().unwrap();
            let exp = exp.elems.iter().collect::<Vec<_>>();
            let h = *exp.first().unwrap();
            let m = *exp.get(1).unwrap();
            let s = *exp.get(2).unwrap();

            if let syn::Type::Path(e) = &field.ty {
                if e.path.segments.iter().any(|x| x.ident == "Option") {
                    return quote! {
                        #field_name: sqlx::Row::get::<'_, Option<time::OffsetDateTime>, _>(&value, stringify!(#field_name)).map(|x| x.to_offset(time::UtcOffset::from_hms(#h, #m, #s).unwrap()))
                    };
                }
            }

            quote! {
                #field_name: sqlx::Row::get::<'_, time::OffsetDateTime, _>(&value, stringify!(#field_name)).to_offset(time::UtcOffset::from_hms(#h, #m, #s).expect(&format!("Invalid offset h: {}, m: {}, s: {}", stringify!(#h), stringify!(#m), stringify!(#s))))
            }
        } else {

            if attrs.iter().any(|x| x.path().is_ident("default")) {
                panic!("Default is used only when the FromStr attribute is present");
            }

            quote! { #field_name: /*value.get(stringify!(#field_name))*/ sqlx::Row::get(&value, stringify!(#field_name)) }
        }

    }).collect::<Vec<_>>();

    quote! {
        impl From<sqlx::postgres::PgRow> for #name {
            fn from(value: sqlx::postgres::PgRow) -> #name {
                #name {
                    #(#fields),*
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(Updatable)]
pub fn create_updatable(token: TokenStream) -> TokenStream {
    let ast = syn::parse(token).unwrap();

    impl_updatable(&ast)
}

fn impl_updatable(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("This isn't a struct"),
    }
    .iter()
    .filter_map(|x| x.ident.as_ref())
    .collect::<Vec<_>>();

    quote! {
        impl crate::database::repository::Updatable for #name {
            fn get_pair(self) -> Option<::std::collections::HashMap<&'static str, crate::database::repository::TypeTable>> {
                let mut resp = ::std::collections::HashMap::new();
                #(
                    if let Some(value) = self.#fields {
                        resp.insert(stringify!(#fields), value.into());
                    }
                )*

                (!resp.is_empty()).then_some(resp)
            }
        }
    }.into()
}

#[proc_macro_derive(MapQuery)]
pub fn map_params(token: TokenStream) -> TokenStream {
    let tmp = syn::parse(token).unwrap();

    impl_map_params(&tmp)
}

fn impl_map_params(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(e) => &e.fields,
        _ => panic!("Only struct"),
    }
    .iter()
    .map(|field| {
        let ty = &field.ty;
        let name = field.ident.as_ref().unwrap();
        if let syn::Type::Path(e) = ty {
            if e.path.segments.iter().any(|x| x.ident == "Option") {
                return quote! {
                    if let ::std::option::Option::Some(e) = self.#name {
                        condition.insert(stringify!(#name), e.into());
                    }
                };
            }
        }

        quote! {
            condition.insert(stringify!(#name), self.#name.into());
        }
    })
    .collect::<Vec<_>>();

    quote! {
        impl crate::database::repository::MapQuery for #name {
            fn get_pairs(self) -> ::std::option::Option<::std::collections::HashMap<&'static str, crate::database::repository::TypeTable>> {
                let mut condition = ::std::collections::HashMap::new();

                #(#fields)*

                (!condition.is_empty()).then_some(condition)
            }
        }
    }.into()
}
