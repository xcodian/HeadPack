use std::collections::VecDeque;

use decode::headpack_decode;
use encode::headpack_encode;
use object::Object;
use serde_json::json;

mod convert;
mod decode;
mod encode;
mod object;

/*

OPTIMIZATIONS I'VE THOUGHT OF:
- map keys can't be anything other than strings, so why bother even sending their class? we only need length!
    - problem: the is odd check wont work anymore

*/

fn main() {
    // for benchmarking
    let _twitter_json = json!({
      "id": 1186275104,
      "id_str": "1186275104",
      "name": "AYUMI",
      "screen_name": "ayuu0123",
      "location": "",
      "description": "元野球部マネージャー❤︎…最高の夏をありがとう…❤︎",
      "url": null,
      "entities": {
        "description": {
          "urls": []
        }
      },
      "protected": false,
      "followers_count": 262,
      "friends_count": 252,
      "listed_count": 0,
      "created_at": "Sat Feb 16 13:40:25 +0000 2013",
      "favourites_count": 235,
      "utc_offset": null,
      "time_zone": null,
      "geo_enabled": false,
      "verified": false,
      "statuses_count": 1769,
      "lang": "en",
      "contributors_enabled": false,
      "is_translator": false,
      "is_translation_enabled": false,
      "profile_background_color": "C0DEED",
      "profile_background_image_url": "http://abs.twimg.com/images/themes/theme1/bg.png",
      "profile_background_image_url_https": "https://abs.twimg.com/images/themes/theme1/bg.png",
      "profile_background_tile": false,
      "profile_image_url": "http://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg",
      "profile_image_url_https": "https://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg",
      "profile_banner_url": "https://pbs.twimg.com/profile_banners/1186275104/1409318784",
      "profile_link_color": "0084B4",
      "profile_sidebar_border_color": "C0DEED",
      "profile_sidebar_fill_color": "DDEEF6",
      "profile_text_color": "333333",
      "profile_use_background_image": true,
      "default_profile": true,
      "default_profile_image": false,
      "following": false,
      "follow_request_sent": false,
      "notifications": false
    });

    let easy_as_pi = json!({ "ImposterIsSus": "v1", "k2": { "k3": "v3" } });

    let test = json!({
      "KEY": ["a", "b", "c"],
    });

    let object = Object::from_json(test);
    let original_json = object.clone().into_json();

    let encoded = headpack_encode(object.clone());

    println!("origina_json = {}", original_json);

    println!(
        "encoded: {} bytes: {}",
        encoded.len(),
        hex::encode(encoded.clone())
    );

    let decoded = headpack_decode(VecDeque::from(encoded));

    let decoded_json = decoded.clone().into_json();

    println!(" decoded: {}", decoded_json);
    println!("equality: {}", original_json == decoded_json)
}
