use crate::keys::{KeyDefinition, KeyOptions};
use crate::struct_name::StructName;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashSet;
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::{Field, LitBool};

#[derive(Clone)]
pub(crate) struct ModelAttributes {
    pub(crate) struct_name: StructName,
    pub(crate) primary_key: Option<KeyDefinition<()>>,
    pub(crate) secondary_keys: HashSet<KeyDefinition<KeyOptions>>,
    pub(crate) do_export_keys: Option<LitBool>,
}

impl ModelAttributes {
    pub(crate) fn primary_key(&self) -> KeyDefinition<()> {
        self.primary_key.clone().expect("Primary key is not set")
    }

    pub(crate) fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("primary_key") {
            let mut key: KeyDefinition<()> = KeyDefinition::new_empty(self.struct_name.clone());
            let content;
            syn::parenthesized!(content in meta.input);

            // Parse the identifier
            let ident: syn::Ident = content.parse()?;
            key.set_function_name(ident);

            // Expect a comma
            content.parse::<syn::Token![->]>()?;

            // Parse the type
            let ty: syn::Type = content.parse()?;
            let ty_string = ty.to_token_stream().to_string();
            key.field_type = Some(ty_string);

            self.primary_key = Some(key);
        } else if meta.path.is_ident("secondary_key") {
            let mut key: KeyDefinition<KeyOptions> =
                KeyDefinition::new_empty(self.struct_name.clone());
            let content;
            syn::parenthesized!(content in meta.input);

            // Parse the identifier
            let ident: syn::Ident = content.parse()?;
            key.set_function_name(ident);

            // Expect a comma
            content.parse::<syn::Token![->]>()?;

            // Parse the type
            let ty: syn::Type = content.parse()?;
            let ty_string = ty.to_token_stream().to_string();
            key.field_type = Some(ty_string);

            // Parse optional flags
            while !content.is_empty() {
                content.parse::<syn::Token![,]>()?;
                let option: syn::Ident = content.parse()?;
                match option.to_string().as_str() {
                    "unique" => key.options.unique = true,
                    "optional" => key.options.optional = true,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            option,
                            "Unknown option for secondary_key, expected 'unique' or 'optional'",
                        ));
                    }
                }
            }

            self.secondary_keys.insert(key);
        } else if meta.path.is_ident("export_keys") {
            self.do_export_keys = Some(meta.value()?.parse()?);
        } else {
            panic!(
                "Unknown attribute: {}",
                meta.path.get_ident().expect("Expected ident")
            );
        }
        Ok(())
    }

    pub(crate) fn parse_field(&mut self, field: &Field) -> Result<()> {
        for attr in &field.attrs {
            if attr.path().is_ident("primary_key") {
                let mut field_type_token_stream = TokenStream::new();
                field.ty.to_tokens(&mut field_type_token_stream);
                let field_type = field_type_token_stream.to_string();
                self.primary_key = Some(KeyDefinition::new_field(
                    self.struct_name.clone(),
                    field
                        .ident
                        .clone()
                        .expect("Parsed field expected to have an ident for primary_key"),
                    field_type,
                    (),
                ));
            } else if attr.path().is_ident("secondary_key") {
                let mut field_type_token_stream = TokenStream::new();
                field.ty.to_tokens(&mut field_type_token_stream);
                let field_type = field_type_token_stream.to_string();
                let mut secondary_options = KeyOptions::default();
                if attr.meta.require_list().is_ok() {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("unique") {
                            secondary_options.unique = true;
                        } else if meta.path.is_ident("optional") {
                            secondary_options.optional = true;
                        } else {
                            panic!("secondary_key support only 'unique' or 'composable'");
                        }
                        Ok(())
                    })?;
                }

                self.secondary_keys.insert(KeyDefinition::new_field(
                    self.struct_name.clone(),
                    field
                        .ident
                        .clone()
                        .expect("Parsed field expected to have an ident for secondary_key"),
                    field_type,
                    secondary_options,
                ));
            }
        }
        Ok(())
    }
}
