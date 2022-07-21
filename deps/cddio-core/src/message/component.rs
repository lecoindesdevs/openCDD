use serenity::{
    model::{
        application::component::{
            Button as SerenityButton, 
            ComponentType as SerenityComponentType,
            SelectMenuOption as SerenitySelectMenuOption,
            SelectMenu as SerenitySelectMenu,
        }
    }, 
    builder::{CreateInputText, CreateSelectMenu, CreateButton, CreateSelectMenuOption, CreateActionRow, CreateInteractionResponseData, CreateInteractionResponse}
};
pub use serenity::model::application::component::{InputTextStyle, ButtonStyle};
pub use serenity::model::channel::ReactionType;

pub trait SetBuilder<'a, B> {
    fn set_builder(self, builder: &'a mut B) -> &'a mut B;
}

pub struct Button {
    pub style: ButtonStyle,
    pub label: Option<String>,
    pub emoji: Option<ReactionType>,
    pub custom_id: Option<String>,
    pub url: Option<String>,
    pub disabled: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            style: ButtonStyle::Primary,
            label: None,
            emoji: None,
            custom_id: None,
            url: None,
            disabled: false,
        }
    }
}
impl SetBuilder<'_, CreateButton> for Button {
    fn set_builder(self, builder: &mut CreateButton) -> &mut CreateButton {
        builder
            .style(self.style)
            .disabled(self.disabled);
        if let Some(label) = self.label {
            builder.label(label);
        }
        if let Some(emoji) = self.emoji {
            builder.emoji(emoji);
        }
        if let Some(url) = self.url {
            builder.url(url);
        }
        builder
    }
}

impl From<Button> for SerenityButton {
    fn from(button: Button) -> Self {
        SerenityButton {
            kind: SerenityComponentType::Button,
            style: button.style,
            label: button.label,
            emoji: button.emoji,
            custom_id: button.custom_id,
            url: button.url,
            disabled: button.disabled,
        }
    }
}

pub struct SelectMenuOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<ReactionType>,
    pub default_selected: bool,
}

impl Default for SelectMenuOption {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: String::new(),
            description: None,
            emoji: None,
            default_selected: false,
        }
    }
}
impl SetBuilder<'_, CreateSelectMenuOption> for SelectMenuOption {
    fn set_builder(self, builder: &mut CreateSelectMenuOption) -> &mut CreateSelectMenuOption {
        builder
            .label(self.label)
            .value(self.value);
        if self.default_selected {
            builder.default_selection(true);
        }
        if let Some(description) = self.description {
            builder.description(description);
        }
        if let Some(emoji) = self.emoji {
            builder.emoji(emoji);
        }
        builder
    }
}

impl From<SelectMenuOption> for SerenitySelectMenuOption {
    fn from(v: SelectMenuOption) -> Self {
        SerenitySelectMenuOption {
            label: v.label,
            value: v.value,
            description: v.description,
            emoji: v.emoji,
            default: v.default_selected,
        }
    }
}

pub struct SelectMenu {
    pub placeholder: Option<String>,
    pub min_values: Option<u64>,
    pub max_values: Option<u64>,
    pub options: Vec<SelectMenuOption>,
}

impl Default for SelectMenu {
    fn default() -> Self {
        Self {
            placeholder: None,
            min_values: None,
            max_values: None,
            options: Vec::new(),
        }
    }
}

impl SetBuilder<'_, CreateSelectMenu> for SelectMenu {
    fn set_builder(self, builder: &mut CreateSelectMenu) -> &mut CreateSelectMenu {
        if let Some(placeholder) = self.placeholder {
            builder.placeholder(placeholder);
        }
        if let Some(min_values) = self.min_values {
            builder.min_values(min_values);
        }
        if let Some(max_values) = self.max_values {
            builder.max_values(max_values);
        }
        builder.options(|options| {
            self.options.into_iter().for_each(|v| {
                options.create_option(|opt| v.set_builder(opt));
            });
            options
        });
        builder
    }
}

impl From<SelectMenu> for SerenitySelectMenu {
    fn from(v: SelectMenu) -> Self {
        SerenitySelectMenu {
            kind: SerenityComponentType::SelectMenu,
            placeholder: v.placeholder,
            custom_id: None,
            min_values: v.min_values,
            max_values: v.max_values,
            options: v.options.into_iter().map(SerenitySelectMenuOption::from).collect(),
            values: Vec::new(),
        }
    }
}


pub struct InputText {
    pub label: String, 
    pub style: InputTextStyle,
    pub placeholder: Option<String>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub default_value: Option<String>,
}

impl Default for InputText {
    fn default() -> Self {
        Self {
            label: String::new(),
            style: InputTextStyle::Short,
            placeholder: None,
            min_length: None,
            max_length: None,
            default_value: None,
        }
    }
}
impl SetBuilder<'_, CreateInputText> for InputText {
    fn set_builder(self, builder: &mut CreateInputText) -> &mut CreateInputText {
        builder.label(self.label);
        builder.style(self.style);
        if let Some(placeholder) = self.placeholder {
            builder.placeholder(placeholder);
        }
        if let Some(min_length) = self.min_length {
            builder.min_length(min_length);
        }
        if let Some(max_length) = self.max_length {
            builder.max_length(max_length);
        }
        if let Some(default_value) = self.default_value {
            builder.value(default_value);
        }
        builder
    }
}

pub enum ComponentType {
    InputText(InputText),
    SelectMenu(SelectMenu),
    Button(Button),
}

pub struct Component {
    pub custom_id: Option<String>,
    pub kind: ComponentType,
}

impl Component {
    pub fn input_text(custom_id: String, input_text: InputText) -> Self {
        Self {
            custom_id: Some(custom_id),
            kind: ComponentType::InputText(input_text),
        }
    }
    pub fn select_menu(custom_id: String, select_menu: SelectMenu) -> Self {
        Self {
            custom_id: Some(custom_id),
            kind: ComponentType::SelectMenu(select_menu),
        }
    }
    pub fn button(custom_id: String, button: Button) -> Self {
        Self {
            custom_id: Some(custom_id),
            kind: ComponentType::Button(button),
        }
    }
}

impl SetBuilder<'_, CreateActionRow> for Component {
    fn set_builder(self, builder: &mut CreateActionRow) -> &mut CreateActionRow {
        match self.kind {
            ComponentType::InputText(v) => {
                builder.create_input_text(|input_text| {
                    if let Some(custom_id) = self.custom_id {
                        input_text.custom_id(custom_id);
                    }
                    v.set_builder(input_text)
                })
            }
            ComponentType::SelectMenu(v) => {
                builder.create_select_menu(|select_menu| {
                    if let Some(custom_id) = self.custom_id {
                        select_menu.custom_id(custom_id);
                    }
                    v.set_builder(select_menu)
                })
            }
            ComponentType::Button(v) => {
                builder.create_button(|button| {
                    if let Some(custom_id) = self.custom_id {
                        button.custom_id(custom_id);
                    }
                    v.set_builder(button)
                })
            }
        }
    }
}

pub struct Modal {
    pub title: String,
    pub custom_id: String,
    pub components: Vec<Component>,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            title: String::new(),
            custom_id: String::new(),
            components: Vec::new(),
        }
    }
}
impl<'a, 'b> SetBuilder<'b, CreateInteractionResponse<'a>> for Modal {
    fn set_builder(self, builder: &'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a> {
        use serenity::model::application::interaction::InteractionResponseType;
        builder
            .kind(InteractionResponseType::Modal)
            .interaction_response_data(|data| self.set_builder(data));
        builder
    }
}
impl<'a, 'b> SetBuilder<'b, CreateInteractionResponseData<'a>> for Modal {
    fn set_builder(self, builder: &'b mut CreateInteractionResponseData<'a>) -> &'b mut CreateInteractionResponseData<'a> {
        builder
            .title(self.title)
            .custom_id(self.custom_id);
        builder.components(|components| {
            self.components.into_iter().for_each(|v| {
                components.create_action_row(|action_row| v.set_builder(action_row));
            });
            components
        });
        builder
    }
}