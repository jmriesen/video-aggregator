use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let videos = use_state(|| vec![]);
    {
        let videos = videos.clone();
        use_effect_with((), move |_| {
            let videos = videos.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_videos: Vec<String> = serde_json::from_str(
                    &reqwest::get("http://localhost:8081/videos")
                        .await
                        .expect("this should be fine")
                        .text()
                        .await
                        .expect("Test"),
                )
                .unwrap();

                videos.set(fetched_videos);
            });
            || ()
        });
    }
    let videos = videos
        .iter()
        .cloned()
        .map(|id| html!(<li style="text-align:center; list-style-type: none;"><Video id= {id}/></li>))
        .collect::<Html>();

    html! {
    <div class="container-sm m-5">
    <ul class="">
    {videos}
    </ul>
    </div>
    }
}

#[derive(Properties, PartialEq)]
struct VideoProps {
    id: String,
}

#[function_component]
fn Video(props: &VideoProps) -> Html {
    let url = format!(
        "https://www.youtube.com/embed/{}?autoplay=0&mute=0",
        props.id
    );

    html!(<iframe  style="width:100%;aspect-ratio: 16 / 9;" src={url} referrerpolicy="strict-origin-when-cross-origin"/>)
}

fn main() {
    yew::Renderer::<App>::new().render();
}
