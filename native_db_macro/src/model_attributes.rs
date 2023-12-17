use crate::keys::{DatabaseKeyDefinition, DatabaseSecondaryKeyOptions};
use crate::struct_name::StructName;
use std::collections::HashSet;
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::Field;

#[derive(Clone)]
pub(crate) struct ModelAttributes {
    pub(crate) struct_name: StructName,
    pub(crate) primary_key: Option<DatabaseKeyDefinition<()>>,
    pub(crate) secondary_keys: HashSet<DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>>,
}

impl ModelAttributes {
    pub(crate) fn primary_key(&self) -> DatabaseKeyDefinition<()> {
        self.primary_key.clone().expect("Primary key is not set")
    }

    pub(crate) fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("primary_key") {
            let mut key: DatabaseKeyDefinition<()> =
                DatabaseKeyDefinition::new_empty(self.struct_name.clone());
            meta.parse_nested_meta(|meta| {
                if key.is_empty() {
                    key.set_function_name(meta.path.get_ident().unwrap().clone());
                } else {
                    panic!(
                        "Unknown attribute: {}",
                        meta.path.get_ident().unwrap().to_string()
                    );
                }
                Ok(())
            })?;
            self.primary_key = Some(key);
        } else if meta.path.is_ident("secondary_key") {
            let mut key: DatabaseKeyDefinition<DatabaseSecondaryKeyOptions> =
                DatabaseKeyDefinition::new_empty(self.struct_name.clone());
            meta.parse_nested_meta(|meta| {
                if key.is_empty() {
                    key.set_function_name(meta.path.get_ident().unwrap().clone());
                } else if meta.path.is_ident("unique") {
                    key.options.unique = true;
                } else if meta.path.is_ident("optional") {
                    key.options.optional = true;
                } else {
                    panic!(
                        "Unknown attribute: {}",
                        meta.path.get_ident().unwrap().to_string()
                    );
                }
                Ok(())
            })?;
            self.secondary_keys.insert(key);
        } else {
            panic!(
                "Unknown attribute: {}",
                meta.path.get_ident().unwrap().to_string()
            );
        }
        Ok(())
    }

    pub(crate) fn parse_field(&mut self, field: &Field) -> Result<()> {
        for attr in &field.attrs {
            if attr.path().is_ident("primary_key") {
                self.primary_key = Some(DatabaseKeyDefinition::new_field(
                    self.struct_name.clone(),
                    field.ident.clone().unwrap(),
                    (),
                ));
            } else if attr.path().is_ident("secondary_key") {
                let mut secondary_options = DatabaseSecondaryKeyOptions::default();
                if let Ok(_) = attr.meta.require_list() {
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

                self.secondary_keys.insert(DatabaseKeyDefinition::new_field(
                    self.struct_name.clone(),
                    field.ident.clone().unwrap(),
                    secondary_options,
                ));
            }
        }
        Ok(())
    }
}
