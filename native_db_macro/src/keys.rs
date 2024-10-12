use crate::struct_name::StructName;
use crate::ToTokenStream;
use quote::quote;
use quote::ToTokens;
use std::hash::Hash;
use syn::PathArguments;
use syn::{parse_str, Ident, Type};

#[derive(Clone, Debug)]
pub(crate) struct KeyDefinition<O: ToTokenStream> {
    pub(super) struct_name: StructName,
    field_name: Option<Ident>,
    function_name: Option<Ident>,
    pub(crate) field_type: Option<String>,
    pub(crate) options: O,
}

impl<O: ToTokenStream> PartialEq for KeyDefinition<O> {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}

impl<O: ToTokenStream> Eq for KeyDefinition<O> {}

impl<O: ToTokenStream> Hash for KeyDefinition<O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident().hash(state);
    }
}

impl<O: ToTokenStream> ToTokenStream for KeyDefinition<O> {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream {
        let options = self.options.new_to_token_stream();
        let struct_name = self.struct_name.ident();
        let key_name = self.name();
        let rust_type_name = self
            .field_type
            .clone()
            .expect("KeyDefinition must have a field type");

        // DEBUG print
        // let rust_type_name: &str = "Vec<u32>";
        // let type_str = "u32";
        let mut parsed_type: Type = parse_str(&rust_type_name).expect("Failed to parse type");

        if let Type::Path(ref mut path, ..) = parsed_type {
            if let Some(segment) = path.path.segments.last_mut() {
                if let PathArguments::AngleBracketed(ref mut args) = segment.arguments {
                    if args.colon2_token.is_none() {
                        let new_args = args.clone();
                        segment.arguments = PathArguments::None;

                        let modified_path: syn::Path = syn::parse_quote! {
                            #path :: #new_args
                        };

                        path.path.segments = modified_path.segments;
                    }
                }
            }
        }

        let parsed_type_token_stream = parsed_type.to_token_stream();

        quote! {
            native_db::db_type::KeyDefinition::new(
                #struct_name::native_model_id(),
                #struct_name::native_model_version(),
                #key_name,
                <#parsed_type_token_stream>::key_names(),
                #options
            )
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct KeyOptions {
    pub(crate) unique: bool,
    pub(crate) optional: bool,
}

impl ToTokenStream for KeyOptions {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream {
        let unique = self.unique;
        let optional = self.optional;
        quote! {
            native_db::db_type::KeyOptions {
                unique: #unique,
                optional: #optional,
            }
        }
    }
}

impl ToTokenStream for () {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream {
        quote! {()}
    }
}

impl<O: ToTokenStream> KeyDefinition<O> {
    pub(crate) fn name(&self) -> String {
        if let Some(field_name) = &self.field_name {
            field_name.to_string().to_lowercase()
        } else if let Some(function_name) = &self.function_name {
            function_name.to_string().to_lowercase()
        } else {
            panic!("Must be either field or function")
        }
    }

    pub(crate) fn ident(&self) -> Ident {
        if self.is_field() {
            self.field_name
                .as_ref()
                .expect("Trying to get an undefined field name")
                .clone()
        } else {
            self.function_name
                .as_ref()
                .expect("Trying to get an undefined function name")
                .clone()
        }
    }

    pub(crate) fn new_field(
        table_name: StructName,
        field_name: Ident,
        field_type: String,
        options: O,
    ) -> Self {
        Self {
            struct_name: table_name,
            field_name: Some(field_name),
            function_name: None,
            field_type: Some(field_type),
            options,
        }
    }

    pub(crate) fn set_function_name(&mut self, function_name: Ident) {
        self.function_name = Some(function_name);
    }

    pub(crate) fn new_empty(table_name: StructName) -> Self
    where
        O: Default,
    {
        Self {
            struct_name: table_name,
            field_name: None,
            function_name: None,
            field_type: None,
            options: O::default(),
        }
    }

    fn check_field_and_function(&self) {
        if self.field_name.is_some() && self.function_name.is_some() {
            panic!("Cannot be both field and function")
        } else if self.field_name.is_none() && self.function_name.is_none() {
            panic!("Must be either field or function")
        }
    }

    pub(crate) fn is_field(&self) -> bool {
        self.check_field_and_function();
        self.field_name.is_some()
    }

    pub(crate) fn is_function(&self) -> bool {
        self.check_field_and_function();
        self.function_name.is_some()
    }

    // TODO: check why this method is not used
    // pub(crate) fn is_empty(&self) -> bool {
    //     self.field_name.is_none() && self.function_name.is_none()
    // }
}
