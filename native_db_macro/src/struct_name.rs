use proc_macro2::Ident;

#[derive(Clone, Debug)]
pub(crate) struct StructName(Ident);

impl StructName {
    pub(crate) fn ident(&self) -> &Ident {
        &self.0
    }
    pub(crate) fn new(ident: Ident) -> Self {
        Self(ident)
    }
}
