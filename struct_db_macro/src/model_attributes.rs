use std::collections::HashSet;
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::Ident;

#[derive(Default, Clone)]
pub(crate) struct ModelAttributes {
    pk_function_name: Option<Ident>, // Primary Key Function Name
    gk_function_names: HashSet<Ident>, // Generic Secondary Key Function Names // gk ou gsk
                                     //  TODO: Derived Secondary Key Function Names: dk ou dsk
}

impl ModelAttributes {
    pub(crate) fn pk(&self) -> Ident {
        self.pk_function_name.clone().expect("pk is required")
    }

    pub(crate) fn pk_name(&self) -> String {
        self.pk().to_string().to_lowercase()
    }

    pub(crate) fn gk_function_names(&self) -> HashSet<Ident> {
        self.gk_function_names.clone()
    }

    pub(crate) fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("pk") {
            self.pk_function_name = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("gk") {
            self.gk_function_names.insert(meta.value()?.parse()?);
        } else {
            panic!(
                "Unknown attribute: {}",
                meta.path.get_ident().unwrap().to_string()
            );
        }
        Ok(())
    }
}
