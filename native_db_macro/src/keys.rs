use crate::struct_name::StructName;
use crate::ToTokenStream;
use quote::quote;
use std::hash::Hash;
use syn::Ident;

#[derive(Clone)]
pub(crate) struct DatabaseKeyDefinition<O: ToTokenStream> {
    pub(super) struct_name: StructName,
    field_name: Option<Ident>,
    function_name: Option<Ident>,
    pub(crate) options: O,
}

impl<O: ToTokenStream> PartialEq for DatabaseKeyDefinition<O> {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}

impl<O: ToTokenStream> Eq for DatabaseKeyDefinition<O> {}

impl<O: ToTokenStream> Hash for DatabaseKeyDefinition<O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident().hash(state);
    }
}

impl<O: ToTokenStream> ToTokenStream for DatabaseKeyDefinition<O> {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream {
        let options = self.options.new_to_token_stream();
        let struct_name = self.struct_name.ident();
        let key_name = self.name();
        quote! {
            native_db::db_type::DatabaseKeyDefinition::new(#struct_name::native_model_id(), #struct_name::native_model_version(), #key_name, #options)
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct DatabaseSecondaryKeyOptions {
    pub(crate) unique: bool,
    pub(crate) optional: bool,
}

impl ToTokenStream for DatabaseSecondaryKeyOptions {
    fn new_to_token_stream(&self) -> proc_macro2::TokenStream {
        let unique = self.unique;
        let optional = self.optional;
        quote! {
            native_db::db_type::DatabaseSecondaryKeyOptions {
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

impl<O: ToTokenStream> DatabaseKeyDefinition<O> {
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
            self.field_name.as_ref().unwrap().clone()
        } else {
            self.function_name.as_ref().unwrap().clone()
        }
    }

    pub(crate) fn new_field(table_name: StructName, field_name: Ident, options: O) -> Self {
        Self {
            struct_name: table_name,
            field_name: Some(field_name),
            function_name: None,
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

    pub(crate) fn is_empty(&self) -> bool {
        self.field_name.is_none() && self.function_name.is_none()
    }
}
