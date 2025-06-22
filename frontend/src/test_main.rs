use yew::prelude::*;

#[function_component(TestComponent)]
pub fn test_component() -> Html {
    html! {
        <div class="test-class">
            {"Hello World"}
        </div>
    }
}

fn main() {
    yew::Renderer::<TestComponent>::new().render();
}
