//! Permission requests from web content: WhatsApp Web may show notifications
//! and use the camera and microphone for calls; everything else is refused.

use gtk::glib;
use webkit::prelude::*;

pub fn decide(view: &webkit::WebView, request: &webkit::PermissionRequest) -> bool {
    let from_whatsapp = view
        .uri()
        .is_some_and(|uri| super::policy::may_load_inside(&uri));
    let allowed = from_whatsapp
        && (request.is::<webkit::NotificationPermissionRequest>()
            || request.is::<webkit::UserMediaPermissionRequest>());
    if allowed {
        request.allow();
    } else {
        request.deny();
        glib::g_debug!(
            "whatsup-desktop",
            "refused a {} permission request",
            request.type_().name()
        );
    }
    true
}

/// Tells WebKit up front that WhatsApp Web may show notifications, so
/// WhatsApp doesn't ask for them again on every start.
pub fn grant_notifications(context: &webkit::WebContext) {
    context.connect_initialize_notification_permissions(|context| {
        let whatsapp = webkit::SecurityOrigin::for_uri(super::WHATSAPP_URL);
        context.initialize_notification_permissions(&[&whatsapp], &[]);
    });
}
