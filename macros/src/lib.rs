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

    let table_name = ast.attrs.iter().find(|x| x.path().is_ident("table_name") ).map(|attr| attr.parse_args::<syn::LitStr>().ok() ).flatten().map(|x| x.value() ).unwrap_or(t.to_string().to_lowercase());

    let fields = match &ast.data {
        Data::Struct(e) => &e.fields,
        _ => panic!("Derive only pemit in structs")
    }.iter().filter_map(|x| {
        x.ident.as_ref()
    }).collect::<Vec<&Ident>>();
    
    quote!{
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
    }.into()
}


