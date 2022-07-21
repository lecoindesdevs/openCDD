use quote::ToTokens;
use syn::spanned::Spanned;
use std::fmt;
use quote::quote;
use crate::util::{MacroArgs, ParenValue};
use super::custom_id::CustomIdAttribute;

use crate::function::Function;

pub struct MessageComponent {
    attr: CustomIdAttribute,
    impl_fn: syn::ImplItemMethod,
}


impl MessageComponent {
    pub fn new(attr: syn::Attribute, impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attr = CustomIdAttribute::from_attr(attr)?;
        Ok(MessageComponent {
            attr,
            impl_fn,
        })
    }
}

impl Function for MessageComponent {
    fn name(&self) -> proc_macro2::TokenStream {
        let name = &self.impl_fn.sig.ident;
        quote! { #name }
    }

    fn event_handle(&self) -> syn::Result<proc_macro2::TokenStream> {
        let func_name = self.name();
        let custom_id = &self.attr.custom_id;
        Ok(quote!{
            serenity::model::event::Event::InteractionCreate(serenity::model::event::InteractionCreateEvent{interaction: serenity::model::application::interaction::Interaction::MessageComponent(message_interaction), ..}) if message_interaction.data.custom_id == #custom_id => self.#func_name(ctx, message_interaction).await
        })
    }
}

impl ToTokens for MessageComponent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_fn.to_tokens(tokens);
    }
}

impl fmt::Debug for MessageComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageComponent")
            .field("custom_id", &self.attr.custom_id)
            .finish()
    }
}