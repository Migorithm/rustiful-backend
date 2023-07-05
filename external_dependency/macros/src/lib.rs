use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Meta, Path};
fn extract_attribute_ident(attr: &Attribute) -> Option<::std::string::String> {
    match attr.meta {
        Meta::Path(Path { ref segments, .. }) => segments.first().map(|s| {
            if format!("{}", s.ident) == *"external" {
                "fn external(&self)->bool{true}".to_string()
            } else if format!("{}", s.ident) == *"internal" {
                "fn internal(&self)->bool{true}".to_string()
            } else {
                panic!("")
            }
        }),
        _ => None,
    }
}

#[proc_macro_derive(Event, attributes(external, internal))]
pub fn derive(input: TokenStream) -> TokenStream {
    // parsing token stream!
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);

    let parsed_attrs = attrs
        .iter()
        .filter_map(extract_attribute_ident)
        .collect::<Vec<_>>();

    if !parsed_attrs.is_empty() {
        let notify: proc_macro2::TokenStream = parsed_attrs.join("").parse().unwrap();

        let a = quote! {
            impl macro_dependency::Message for #ident {
                #notify
                fn metadata(&self) -> macro_dependency::MessageMetadata {
                    macro_dependency::MessageMetadata {
                        aggregate_id: self.id.to_string(),
                        topic: stringify!(#ident).into(),
                    }
                }
                fn message_clone(&self)-> Box<dyn macro_dependency::Message>{
                    Box::new(self.clone())
                }
                fn state(&self) -> String {
                    serde_json::to_string(&self).expect("Failed to serialize")
                }
            }
        };
        println!("{}", a.to_string());
        a.into()
    } else {
        quote! {}.into()
    }
}
