use simpleml_macro::sml;

fn main() {
    let tree = sml!{
        Configuration
            Video
                Resolution 1280 720
                RefreshRate   60
                Fullscreen true
            my_custom_end_keyword
            Audio
                Volume 100
                Music  80
            my_custom_end_keyword
            Player
                Name "Hero 123"
            my_custom_end_keyword
        my_custom_end_keyword
    };
    println!("{tree:?}")
}
