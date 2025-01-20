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
        .and_then(|attr| attr.parse_args::<syn::LitStr>().map(|x| x.value() ).ok())
        .unwrap_or(t.to_string().to_lowercase());

    let fields = match &ast.data {
        Data::Struct(e) => &e.fields,
        _ => panic!("Derive only pemit in structs"),
    }
    .iter()
    .filter_map(|x| x.ident.as_ref())
    .collect::<Vec<&Ident>>();

    quote! {
        impl Table for #t {
            fn name() -> String {
                #table_name.to_string()
            }

            fn get_fields(self) -> Vec<TypeTable> {
                let mut resp = Vec::new();
                #(
                    resp.push(self.#fields.into());
                )*
                resp
            }

            fn columns() -> Vec<&'static str> {
                let mut resp = Vec::new();
                #(
                    resp.push(stringify!(#fields));
                )*

                resp
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromPgRow, attributes(FromStr, default))]
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
        
        let ty = field.ty.clone();
        if attrs.iter().any(|x| x.path().is_ident("FromStr")) {
            if attrs.iter().any(|x|x.path().is_ident("default")) {
                quote! { #field_name: value.get::<'_, &str, _>(stringify!(#field_name)).parse::<#ty>().unwrap_or_default() }
            } else {
                quote! { #field_name: value.get::<'_, &str, _>(stringify!(#field_name)).parse::<#ty>().unwrap() }
            }
        } else {
            quote! { #field_name: value.get(stringify!(#field_name)) }
        }

    }).collect::<Vec<_>>();

    quote! {
        impl From<PgRow> for #name {
            fn from(value: PgRow) -> #name {
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
    }.iter().filter_map(|x| x.ident.as_ref() ).collect::<Vec<_>>();

    quote! {
        impl<'a> Updatable<'a> for #name {
            fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
                let mut resp = HashMap::new();
                #(
                    if let Some(value) = self.#fields {
                        resp.insert(stringify!(#fields), value.into());
                    }
                )*

                if !resp.is_empty() {
                    Some(resp)
                } else {
                    None
                }
            }
        }
    }.into()
}