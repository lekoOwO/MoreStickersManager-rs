use std::sync::Mutex;

use async_trait::async_trait;
use msm_telegram::{
    publish_sticker_set, TelegramPublishRequest, TelegramPublishSticker, TelegramPublishedSet,
    TelegramStickerSetApi,
};
use teloxide::types::{InputFile, InputSticker, StickerFormat, StickerType};

#[tokio::test]
async fn publish_sticker_set_creates_then_appends_in_order() {
    let api = RecordingTelegramApi::default();
    let result = publish_sticker_set(
        &api,
        TelegramPublishRequest {
            owner_user_id: 42,
            sticker_set_name: "cats_by_msm_bot".to_owned(),
            title: "Cats".to_owned(),
            sticker_type: StickerType::Regular,
            initial_stickers: vec![publish_sticker("cat_1")],
            append_stickers: vec![publish_sticker("cat_2"), publish_sticker("cat_3")],
        },
    )
    .await
    .unwrap();

    assert_eq!(
        result,
        TelegramPublishedSet {
            sticker_set_name: "cats_by_msm_bot".to_owned(),
            title: "Cats".to_owned(),
            sticker_count: 3,
            url: "https://t.me/addstickers/cats_by_msm_bot".to_owned(),
        }
    );
    assert_eq!(
        api.calls.lock().unwrap().as_slice(),
        [
            "create:42:cats_by_msm_bot:Cats:1:regular",
            "append:42:cats_by_msm_bot:cat_2",
            "append:42:cats_by_msm_bot:cat_3",
        ]
    );
}

#[derive(Default)]
struct RecordingTelegramApi {
    calls: Mutex<Vec<String>>,
}

#[async_trait]
impl TelegramStickerSetApi for RecordingTelegramApi {
    async fn create_new_sticker_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        title: &str,
        sticker_type: StickerType,
        stickers: Vec<TelegramPublishSticker>,
    ) -> Result<(), msm_telegram::TelegramPublishError> {
        let sticker_type = match sticker_type {
            StickerType::Regular => "regular",
            StickerType::Mask => "mask",
            StickerType::CustomEmoji => "custom_emoji",
        };
        self.calls.lock().unwrap().push(format!(
            "create:{owner_user_id}:{sticker_set_name}:{title}:{}:{sticker_type}",
            stickers.len()
        ));
        Ok(())
    }

    async fn add_sticker_to_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        sticker: TelegramPublishSticker,
    ) -> Result<(), msm_telegram::TelegramPublishError> {
        self.calls.lock().unwrap().push(format!(
            "append:{owner_user_id}:{sticker_set_name}:{}",
            sticker.source_sticker_id
        ));
        Ok(())
    }
}

fn publish_sticker(id: &str) -> TelegramPublishSticker {
    TelegramPublishSticker {
        source_sticker_id: id.to_owned(),
        input: InputSticker {
            sticker: InputFile::memory(vec![1, 2, 3]).file_name(format!("{id}.png")),
            format: StickerFormat::Static,
            emoji_list: vec!["🐱".to_owned()],
            mask_position: None,
            keywords: vec![id.to_owned()],
        },
    }
}
