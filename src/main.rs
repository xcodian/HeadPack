use std::{collections::VecDeque, f32::consts::PI};

use decode::headpack_message_decode;
use encode::headpack_encode;
use object::Object;
use serde_json::json;

mod convert;
mod decode;
mod encode;
mod object;

fn main() {
    // old data = 93c04e3762636f6d706163745768656e546865496d706f737465724973537573706963696f757300
    // older data = 93c04e3c20636f6d70616374736368656d6100

    // let data = VecDeque::from(
    //     hex::decode(
    //         "93c04e3762636f6d706163745768656e546865496d706f737465724973537573706963696f7573c7a4",
    //     )
    //     .unwrap(),
    // );
    // // data.con

    // // let objects = decode(data);

    let raw = json!({
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

    /*

    OPTIMIZATIONS I'VE THOUGHT OF:
    - map keys can't be anything other than strings, so why bother even sending their class? we only need length!
        - problem: the is odd check wont work anymore
     */

    // let benchmark = Object::map(vec![
    //     (Object::string("id".to_string()), Object::sint(1186275104)),
    //     (
    //         Object::string("id_str".to_string()),
    //         Object::string("1186275104".to_string()),
    //     ),
    //     (
    //         Object::string("name".to_string()),
    //         Object::string("AYUMI".to_string()),
    //     ),
    //     (
    //         Object::string("screen_name".to_string()),
    //         Object::string("ayuu0123".to_string()),
    //     ),
    //     (
    //         Object::string("location".to_string()),
    //         Object::string("".to_string()),
    //     ),
    //     (
    //         Object::string("description".to_string()),
    //         Object::string("元野球部マネージャー❤︎…最高の夏をありがとう…❤︎".to_string()),
    //     ),
    //     (Object::string("url".to_string()), Object::null()),
    //     (
    //         Object::string("entities".to_string()),
    //         Object::map(vec![(
    //             Object::string("description".to_string()),
    //             Object::map(vec![(
    //                 Object::string("urls".to_string()),
    //                 Object::list(vec![]),
    //             )]),
    //         )]),
    //     ),
    //     (Object::string("protected".to_string()), Object::bool(false)),
    //     (
    //         Object::string("followers_count".to_string()),
    //         Object::sint(262),
    //     ),
    //     (
    //         Object::string("friends_count".to_string()),
    //         Object::sint(252),
    //     ),
    //     (Object::string("listed_count".to_string()), Object::sint(0)),
    //     (
    //         Object::string("created_at".to_string()),
    //         Object::string("Sat Feb 16 13:40:25 +0000 2013".to_string()),
    //     ),
    //     (
    //         Object::string("favourites_count".to_string()),
    //         Object::sint(235),
    //     ),
    //     (Object::string("utc_offset".to_string()), Object::null()),
    //     (Object::string("time_zone".to_string()), Object::null()),
    //     (
    //         Object::string("geo_enabled".to_string()),
    //         Object::bool(false),
    //     ),
    //     (Object::string("verified".to_string()), Object::bool(false)),
    //     (
    //         Object::string("statuses_count".to_string()),
    //         Object::sint(1769),
    //     ),
    //     (
    //         Object::string("lang".to_string()),
    //         Object::string("en".to_string()),
    //     ),
    //     (
    //         Object::string("contributors_enabled".to_string()),
    //         Object::bool(false),
    //     ),
    //     (
    //         Object::string("is_translator".to_string()),
    //         Object::bool(false),
    //     ),
    //     (
    //         Object::string("is_translation_enabled".to_string()),
    //         Object::bool(false),
    //     ),
    //     (
    //         Object::string("profile_background_color".to_string()),
    //         Object::string("C0DEED".to_string()),
    //     ),
    //     (
    //         Object::string("profile_background_image_url".to_string()),
    //         Object::string("http://abs.twimg.com/images/themes/theme1/bg.png".to_string()),
    //     ),
    //     (
    //         Object::string("profile_background_image_url_https".to_string()),
    //         Object::string("https://abs.twimg.com/images/themes/theme1/bg.png".to_string()),
    //     ),
    //     (
    //         Object::string("profile_background_tile".to_string()),
    //         Object::bool(false),
    //     ),
    //     (
    //         Object::string("profile_image_url".to_string()),
    //         Object::string(
    //             "http://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg"
    //                 .to_string(),
    //         ),
    //     ),
    //     (
    //         Object::string("profile_image_url_https".to_string()),
    //         Object::string(
    //             "https://pbs.twimg.com/profile_images/497760886795153410/LDjAwR_y_normal.jpeg"
    //                 .to_string(),
    //         ),
    //     ),
    //     (
    //         Object::string("profile_banner_url".to_string()),
    //         Object::string(
    //             "https://pbs.twimg.com/profile_banners/1186275104/1409318784".to_string(),
    //         ),
    //     ),
    //     (
    //         Object::string("profile_link_color".to_string()),
    //         Object::string("0084B4".to_string()),
    //     ),
    //     (
    //         Object::string("profile_sidebar_border_color".to_string()),
    //         Object::string("C0DEED".to_string()),
    //     ),
    //     (
    //         Object::string("profile_sidebar_fill_color".to_string()),
    //         Object::string("DDEEF6".to_string()),
    //     ),
    //     (
    //         Object::string("profile_text_color".to_string()),
    //         Object::string("333333".to_string()),
    //     ),
    //     (
    //         Object::string("profile_use_background_image".to_string()),
    //         Object::bool(true),
    //     ),
    //     (
    //         Object::string("default_profile".to_string()),
    //         Object::bool(true),
    //     ),
    //     (
    //         Object::string("default_profile_image".to_string()),
    //         Object::bool(false),
    //     ),
    //     (Object::string("following".to_string()), Object::bool(false)),
    //     (
    //         Object::string("follow_request_sent".to_string()),
    //         Object::bool(false),
    //     ),
    //     (
    //         Object::string("notifications".to_string()),
    //         Object::bool(false),
    //     ),
    // ]);

    // let benchmark2 = Object::map(vec![(
    //     Object::string("entities".to_string()),
    //     Object::map(vec![
    //         (
    //             Object::string("url".to_string()),
    //             Object::map(vec![(
    //                 Object::string("urls".to_string()),
    //                 Object::list(vec![Object::map(vec![
    //                     (
    //                         Object::string("url".to_string()),
    //                         Object::string("http://t.co/Yg9e1Fl8wd".to_string()),
    //                     ),
    //                     (
    //                         Object::string("expanded_url".to_string()),
    //                         Object::string("http://twpf.jp/yuttari1998".to_string()),
    //                     ),
    //                     (
    //                         Object::string("display_url".to_string()),
    //                         Object::string("twpf.jp/yuttari1998".to_string()),
    //                     ),
    //                     (
    //                         Object::string("indices".to_string()),
    //                         Object::list(vec![Object::sint(0), Object::sint(22)]),
    //                     ),
    //                 ])]),
    //             )]),
    //         ),
    //         (
    //             Object::string("description".to_string()),
    //             Object::map(vec![(
    //                 Object::string("urls".to_string()),
    //                 Object::list(vec![]),
    //             )]),
    //         ),
    //     ]),
    // )]);

    let messagepack_sample: Object = vec![
        ("easy".into(), true.into()),
        ("as".into(), vec![("pi".into(), PI.into())].into()),
    ]
    .into();

    let from_json = Object::from_json(json!({"easy":true,"as":{"pi":-3.1415927}}));

    let msg = headpack_encode(from_json.clone());
    println!("ALL: {} bytes: {}", msg.len(), hex::encode(msg.clone()));

    let dec = headpack_message_decode(VecDeque::from(msg));

    println!("orig: {:?}", from_json);
    println!("decd: {:?}", dec);

    // println!("{:?} {:?}", sint_to_bytes(27), sint_to_bytes(-13));

    // for i in i128::MIN..i128::MAX {
    //     // println!("{:?} === {:?}", i.to_be_bytes(), (i as i128).to_be_bytes());

    //     let bytes = sint_to_bytes(i);
    //     let j = sint_from_bytes(&bytes);

    //     if i != j {
    //         println!("{} != {}", i, j);
    //     } else {
    //         // println!("pass {}", i);
    //     }
    // }

    // println!("fin");

    // assert_eq!(sint_from_bytes(&sint_to_bytes(0)), 0);

    // let u: u128 = 123456789;
    // let s: i128 = -123456789;

    // let ub = uint_to_bytes(u);
    // let sb = sint_to_bytes(s);

    // println!("uint {} -> {} ({ub:?})", u, uint_from_bytes(&ub));
    // println!("sint {} -> {} ({sb:?})", s, sint_from_bytes(&sb));
}
