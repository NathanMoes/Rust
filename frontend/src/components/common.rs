use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AlertProps {
    pub message: String,
    pub error: bool,
}

#[function_component(Alert)]
pub fn alert(props: &AlertProps) -> Html {
    let base_classes = "p-4 rounded-lg border";
    let variant_classes = if props.error {
        "bg-red-50 border-red-200 text-red-800"
    } else {
        "bg-green-50 border-green-200 text-green-800"
    };

    html! {
        <div class={format!("{} {}", base_classes, variant_classes)}>
            {&props.message}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    pub message: Option<String>,
}

#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div class="flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
            if let Some(message) = &props.message {
                <span class="ml-3 text-gray-600">{message}</span>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub title: String,
    pub children: Children,
}

#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    html! {
        <div class="bg-white rounded-lg shadow-md p-6">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">{&props.title}</h3>
            {for props.children.iter()}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub children: Children,
    pub disabled: Option<bool>,
    pub variant: Option<String>,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let mut classes = "px-4 py-2 rounded-md font-medium transition-colors".to_string();
    
    match props.variant.as_deref() {
        Some("primary") => classes.push_str(" bg-purple-600 text-white hover:bg-purple-700 disabled:bg-purple-300"),
        Some("secondary") => classes.push_str(" bg-gray-200 text-gray-800 hover:bg-gray-300 disabled:bg-gray-100"),
        Some("danger") => classes.push_str(" bg-red-600 text-white hover:bg-red-700 disabled:bg-red-300"),
        _ => classes.push_str(" bg-purple-600 text-white hover:bg-purple-700 disabled:bg-purple-300"),
    }

    html! {
        <button 
            class={classes}
            onclick={&props.onclick}
            disabled={props.disabled.unwrap_or(false)}
        >
            {for props.children.iter()}
        </button>
    }
}
