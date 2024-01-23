use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = ast.ident;
    let data = ast.data;
    // CommandBuilder
    let builder_name = Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    let fields = match data {
        Data::Enum(_) => panic!("The macro only supports structs, not enums"),
        Data::Union(_) => panic!("The macro only supports structs, not unions"),
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields_named) => fields_named.named,
                _ => panic!("This macro only supports structs with named fields"), // Fields::Unnamed(_) => unimplemented!()
            }
        }
    };

    let builder_fields = fields.clone().into_iter().map(|field| {
        let field_name = field.ident.expect("Expected named field");
        quote! {
            #field_name: None
        }
    });

    let builder_struct_fields = fields.clone().into_iter().map(|field| {
        let field_name = field.ident.expect("Expected named field");
        let ty = field.ty;
        quote! {
            #field_name: Option<#ty>
        }
    });

    let setters = fields.clone().into_iter().map(|field| {
        let field_name = field.ident.expect("Expected named field");
        let ty = field.ty;
        quote! {
            fn #field_name(&mut self, #field_name: #ty) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    let build_fn_body = {
        let inner = fields.clone().into_iter().map(|field| {
            let field_name = field.ident.expect("Expected named field");
            // let ty = field.ty;

            quote! {
                #field_name: self.#field_name.clone().ok_or(format!("{} is missing", stringify!(#field_name)))?,
            }
        });
        quote! {
            Ok(#struct_name {
                #(#inner)*
            })
        }
    };

    // Get the fields from the input struct.

    // eprintln!("builder_name: {:?}", builder_name);
    let expanded = quote! {
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_fields),*
                }
            }
        }

        pub struct #builder_name {
            #(#builder_struct_fields),*
        }

        impl #builder_name {
            #(#setters)*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                #build_fn_body
            }
        }
    };

    // expanded.into()
    TokenStream::from(expanded)
}
