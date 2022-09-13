
pub(crate) fn set_global_template_bold_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();

    let bold_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description bold")
        .callback_data("/set_global_template {{bold bot_item_description }}")
        .build();
    let bold_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name bold")
        .switch_inline_query_current_chat("/set_global_template {{bold bot_item_name }}")
        .build();

    row.push(bold_bot_description);
    row2.push(bold_bot_item_name);

    keyboard.push(row);
    keyboard.push(row2);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}


// Update { update_id: 95969982, 
//     content: Message(Message {

//          message_id: 3629, from: Some(User 
//      {
//              id: 614443505, is_bot: false, first_name: 
//              "athul", last_name: Some("krishna"), username: Some("athul_07_07"), 
//              language_code: Some("en"), is_premium: None, added_to_attachment_menu:
//               None, can_join_groups: None, 
//               can_read_all_group_messages: None, 
//               supports_inline_queries: None }), 
//               sender_chat: None, date: 1663048716, 
//               chat: Chat { id: 614443505, 
//                 type_field: Private, title: None,
//                  username: Some("athul_07_07"), 
//                  first_name: Some("athul"), 
//                  last_name: Some("krishna"), 
//                  photo: None, bio: None, 
//                  has_private_forwards: None,
//                   has_restricted_voice_and_video_messages: None,
//                    join_to_send_messages: None, 
//                    join_by_request: None, 
//                    description: None, 
//                    invite_link: None, pinned_message: None, 
//                    permissions: None, slow_mode_delay: None, 
//                    message_auto_delete_time: None, has_protected_content: None, 
//                    sticker_set_name: None, can_set_sticker_set: None, 
//                    linked_chat_id: None, location: None }, 
//                    forward_from: None, forward_from_chat: None, 
//                    forward_from_message_id: None, 
//                    forward_signature: None, forward_sender_name: None, 
//                    forward_date: None, is_automatic_forward: None, 
//                    reply_to_message: None, via_bot: None, 
//                    edit_date: None, has_protected_content: None, 
//                    media_group_id: None, author_signature: None, 
//                    text: Some("@sasaathulbot /set_global_template {{bold bot_item_name }}"),
//                    entities: Some([MessageEntity { type_field: Mention, 
//                     offset: 0, length: 13, url: None, user: None, 
//                     language: None, custom_emoji_id: None }, 
//                     MessageEntity { type_field: BotCommand, offset: 14, 
//                         length: 20, url: None, user: None, language: None, 
//                         custom_emoji_id: None }]), animation: None, 
//                         audio: None, document: None, photo: None, sticker: None, 
//                         video: None, video_note: None, voice: None, caption: None, 
//                         caption_entities: None, contact: None, dice: None, game: None, 
//                         poll: None, venue: None, location: None, new_chat_members: None, 
//                         left_chat_member: None, new_chat_title: None, new_chat_photo: None, 
//                         delete_chat_photo: None, group_chat_created: None, 
//                         supergroup_chat_created: None, channel_chat_created: None, 
//                         message_auto_delete_timer_changed: None, migrate_to_chat_id: None, 
//                         migrate_from_chat_id: None, pinned_message: None, invoice: None, 
//                         successful_payment: None, connected_website: None, passport_data: None,
//                          proximity_alert_triggered: None, video_chat_started: None, 
//                          video_chat_ended: None, video_chat_scheduled: None, 
//                          video_chat_participants_invited: None, web_app_data: None, 
//                          reply_markup: None 
//                         }
//                     ) 
                
// }