use std::sync::Mutex;

use async_trait::async_trait;
use msm_telegram::{
    apply_sticker_set_mutations, fetch_sticker_set, publish_sticker_set, TelegramBotConfig,
    TelegramFetchedSticker, TelegramFetchedStickerSet, TelegramPublishRequest,
    TelegramPublishSticker, TelegramPublishedSet, TelegramStickerSetApi,
    TelegramStickerSetMutation, TeloxideTelegramStickerSetApi,
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

#[test]
fn teloxide_adapter_can_be_built_from_bot_config() {
    let bot = TelegramBotConfig::new("123456:secret").unwrap().build_bot();
    let adapter = TeloxideTelegramStickerSetApi::new(bot);

    assert_api_trait(&adapter);
}

#[tokio::test]
async fn apply_sticker_set_mutations_executes_remote_changes_in_order() {
    let api = RecordingTelegramApi::default();
    let applied = apply_sticker_set_mutations(
        &api,
        vec![
            TelegramStickerSetMutation::SetTitle {
                sticker_set_name: "cats_by_msm_bot".to_owned(),
                title: "Cats v2".to_owned(),
            },
            TelegramStickerSetMutation::ReplaceSticker {
                owner_user_id: 42,
                sticker_set_name: "cats_by_msm_bot".to_owned(),
                old_telegram_file_id: "tg_old".to_owned(),
                sticker: publish_sticker("cat_1"),
            },
            TelegramStickerSetMutation::AddSticker {
                owner_user_id: 42,
                sticker_set_name: "cats_by_msm_bot".to_owned(),
                sticker: publish_sticker("cat_2"),
            },
            TelegramStickerSetMutation::DeleteSticker {
                telegram_file_id: "tg_remote_only".to_owned(),
            },
        ],
    )
    .await
    .unwrap();

    assert_eq!(applied, 4);
    assert_eq!(
        api.calls.lock().unwrap().as_slice(),
        [
            "title:cats_by_msm_bot:Cats v2",
            "replace:42:cats_by_msm_bot:tg_old:cat_1",
            "append:42:cats_by_msm_bot:cat_2",
            "delete:tg_remote_only",
        ]
    );
}

#[tokio::test]
async fn fetch_sticker_set_returns_remote_state_through_api_boundary() {
    let api = RecordingTelegramApi::with_remote_set(TelegramFetchedStickerSet {
        sticker_set_name: "cats_by_msm_bot".to_owned(),
        title: "Cats".to_owned(),
        sticker_type: StickerType::Regular,
        stickers: vec![TelegramFetchedSticker {
            telegram_file_id: "tg_file".to_owned(),
            telegram_file_unique_id: "tg_unique".to_owned(),
            emoji: Some("🐱".to_owned()),
            is_animated: false,
            is_video: false,
        }],
    });

    let remote = fetch_sticker_set(&api, "cats_by_msm_bot").await.unwrap();

    assert_eq!(remote.sticker_set_name, "cats_by_msm_bot");
    assert_eq!(remote.title, "Cats");
    assert_eq!(remote.stickers[0].telegram_file_id, "tg_file");
    assert_eq!(
        api.calls.lock().unwrap().as_slice(),
        ["fetch:cats_by_msm_bot"]
    );
}

#[derive(Default)]
struct RecordingTelegramApi {
    calls: Mutex<Vec<String>>,
    remote_set: Mutex<Option<TelegramFetchedStickerSet>>,
}

impl RecordingTelegramApi {
    fn with_remote_set(remote_set: TelegramFetchedStickerSet) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            remote_set: Mutex::new(Some(remote_set)),
        }
    }
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

    async fn set_sticker_set_title(
        &self,
        sticker_set_name: &str,
        title: &str,
    ) -> Result<(), msm_telegram::TelegramPublishError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("title:{sticker_set_name}:{title}"));
        Ok(())
    }

    async fn replace_sticker_in_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        old_telegram_file_id: &str,
        sticker: TelegramPublishSticker,
    ) -> Result<(), msm_telegram::TelegramPublishError> {
        self.calls.lock().unwrap().push(format!(
            "replace:{owner_user_id}:{sticker_set_name}:{old_telegram_file_id}:{}",
            sticker.source_sticker_id
        ));
        Ok(())
    }

    async fn delete_sticker_from_set(
        &self,
        telegram_file_id: &str,
    ) -> Result<(), msm_telegram::TelegramPublishError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("delete:{telegram_file_id}"));
        Ok(())
    }

    async fn get_sticker_set(
        &self,
        sticker_set_name: &str,
    ) -> Result<TelegramFetchedStickerSet, msm_telegram::TelegramPublishError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("fetch:{sticker_set_name}"));
        self.remote_set.lock().unwrap().clone().ok_or_else(|| {
            msm_telegram::TelegramPublishError::Api {
                message: "missing remote set".to_owned(),
            }
        })
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

fn assert_api_trait<T: TelegramStickerSetApi>(_api: &T) {}
