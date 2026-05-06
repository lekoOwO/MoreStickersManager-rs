# Decisions

## 2026-05-02: Phase-Based Delivery

MSM is implemented in small phases. P0/P1 builds foundation and compatibility only. Later phases require their own specs and plans.

## 2026-05-02: Domain Crate Boundary

`msm-domain` owns compatibility models and pure helper logic. It does not depend on API, database, provider SDK, or frontend crates.

## 2026-05-02: External Format Stability

MoreStickers-compatible JSON is the external contract. Internal data may become richer later, but exports must preserve compatibility.

## 2026-05-06: Provider And Export Target Separation

Providers are input-side normalizers. Export targets are output-side serializers
or remote publishers. Telegram can exist on both sides, but Telegram import must
stay in provider code while Telegram sticker set creation belongs to the planned
export pipeline.
