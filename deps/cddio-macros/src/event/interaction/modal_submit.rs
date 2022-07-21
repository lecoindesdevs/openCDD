use quote::ToTokens;
use std::fmt;
use quote::quote;
use super::custom_id::CustomIdAttribute;

use crate::function::Function;

pub struct ModalSubmit {
    attr: CustomIdAttribute,
    impl_fn: syn::ImplItemMethod,
}

impl ModalSubmit {
    pub fn new(attr: syn::Attribute, impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attr = CustomIdAttribute::from_attr(attr)?;
        Ok(ModalSubmit {
            attr,
            impl_fn,
        })
    }
}

impl Function for ModalSubmit {
    fn name(&self) -> proc_macro2::TokenStream {
        let name = &self.impl_fn.sig.ident;
        quote! { #name }
    }

    fn event_handle(&self) -> syn::Result<proc_macro2::TokenStream> {
        let func_name = self.name();
        let custom_id = &self.attr.custom_id;
        Ok(quote!{
            serenity::model::event::Event::InteractionCreate(serenity::model::event::InteractionCreateEvent{interaction: serenity::model::application::interaction::Interaction::ModalSubmit(modal_submit), ..}) if modal_submit.data.custom_id == #custom_id => self.#func_name(ctx, modal_submit).await
        })
    }
}

impl ToTokens for ModalSubmit {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_fn.to_tokens(tokens);
    }
}

impl fmt::Debug for ModalSubmit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModalSubmit")
            .field("custom_id", &self.attr.custom_id)
            .finish()
    }
}