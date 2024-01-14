use std::{collections::VecDeque, f32::consts::PI};

use decode::headpack_decode;
use encode::headpack_encode;
use object::Object;
use serde_json::{json, Value};

mod convert;
mod decode;
mod encode;
mod object;

fn main() {
    // for benchmarking
    let easy_as_pi = json!({"easy": true, "as": {"pi": PI }});

    let messagepack_demo = json!({"compact": true, "schema": 0});

    let twitter_json = json!({
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

    let dummyjson = json!(
        {
            "id": 10,
            "title": "HP Pavilion 15-DK1056WM",
            "description": "HP Pavilion 15-DK1056WM Gaming...",
            "price": 1099,
            "discountPercentage": 6.18,
            "rating": 4.43,
            "stock": 89,
            "brand": "HP Pavilion",
            "category": "laptops",
            "thumbnail": "https://cdn.dummyjson.com/product-images/10/thumbnail.jpeg",
            "images": [
              "https://cdn.dummyjson.com/product-images/10/1.jpg",
              "https://cdn.dummyjson.com/product-images/10/2.jpg",
              "https://cdn.dummyjson.com/product-images/10/3.jpg",
              "https://cdn.dummyjson.com/product-images/10/thumbnail.jpeg"
            ]
          }
    );

    let zling = json!(
        {
            "avatar": "/media/9ybevZcdBh-3Z2KRLBidT/avatar.png",
            "bot": "false",
            "email": "someone@example.com",
            "id": "xoKM4W7NDqHjK_V0g9s3y",
            "name": "someone#1234"
        }
    );

    let zling2 = json!(
        [
            {
                "attachments": [
                {
                    "id": "s6NIiu2oOh1FEL0Xfjc7n",
                    "name": "cat.jpg",
                    "type": "image",
                    "url": "/media/s6NIiu2oOh1FEL0Xfjc7n/cat.jpg"
                }
                ],
                "author": {
                "avatar": "/media/9ybevZcdBh-3Z2KRLBidT/avatar.png",
                "id": "xoKM4W7NDqHjK_V0g9s3y",
                "username": "someone#1234"
                },
                "content": "Good morning!",
                "createdAt": "1970-01-01T00:00:00.000Z",
                "id": "K1vqjuY8OqU0VO7oJlGpY"
            },
        ]
    );

    benchmark(easy_as_pi);
    benchmark(messagepack_demo);
    benchmark(twitter_json);
    benchmark(dummyjson);
    benchmark(zling);
    benchmark(zling2);
}

fn benchmark(json_object: Value) {
    println!("-------------------------");
    let js = json_object.clone().to_string().len();
    print!("       JSON: {} bytes", js);

    if js <= 200 {
        println!(" {}", json_object.clone());
    } else {
        println!(" (too big to show)");
    }

    let mp = benchmark_messagepack(json_object.clone());
    let hp = benchmark_headpack(json_object.clone());

    let delta_mp = 100.0 - (hp as f32 / mp as f32 * 100.0);
    println!("\nHeadPack is {:.2}% smaller than MessagePack", delta_mp);

    let delta_js = 100.0 - (hp as f32 / js as f32 * 100.0);
    println!("HeadPack is {:.2}% smaller than JSON", delta_js);
    println!("-------------------------");
}

fn benchmark_headpack(json_object: Value) -> usize {
    let object = Object::from_json(json_object.clone());

    let encoded = headpack_encode(object.clone());

    let size = encoded.len();
    println!("   HeadPack: {} bytes", size);

    let decoded = headpack_decode(VecDeque::from(encoded)).into_json();

    println!("expected: {}", json_object);
    println!("     got: {}", decoded);
    if json_object != decoded {
        println!("ERROR: HeadPack failed to decode correctly! :(");
    }

    return size;
}

fn benchmark_messagepack(json_object: Value) -> usize {
    let encoded = rmp_serde::to_vec(&json_object).unwrap();
    let size = encoded.len();
    println!("MessagePack: {} bytes", size);

    let decoded: Value = rmp_serde::from_slice(&encoded).unwrap();

    if json_object != decoded {
        println!("ERROR: MessagePack failed to decode correctly!");
    }

    return size;
}
