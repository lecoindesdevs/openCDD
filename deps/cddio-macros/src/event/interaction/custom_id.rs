use syn::spanned::Spanned;

use crate::util::{ParenValue, MacroArgs};

#[derive(Debug, Clone, Default)]
pub struct CustomIdAttribute{
    pub custom_id: String
}

impl CustomIdAttribute {
    pub fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        let attr_span = attr.span();
        let mut result = Self::default();
        let args = parse2::<ParenValue<MacroArgs>>(attr.tokens)?;
        for arg in args.value.args.into_iter() {
            match (arg.name.to_string().as_str(), arg.value) {
                ("custom_id", Lit::Str(s)) => result.custom_id = s.value(),
                _ => return Err(Error::new_spanned(arg.name, "Argument inconnu ou mal typé.")),
            }
        }
        if result.custom_id.is_empty() {
            return Err(Error::new(attr_span, "Argument custom_id manquant"));
        }
        Ok(result)
    }
}