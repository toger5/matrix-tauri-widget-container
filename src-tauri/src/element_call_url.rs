use urlencoding::encode;
pub static BASE_URL: &str = "https://pr1348--element-call.netlify.app"; //"https://pr1348--element-call.netlify.app"; //"https://call.element.io"; // "https://call.element.dev";

pub fn url(room_id: &str, user_id: &str, widget_id: &str) -> String {
    return format!("{base}/room?widgetId={widget_id}&parentUrl={parent}&embed=&hideHeader=&userId={user}&deviceId=ONLDUUSMTR&roomId={room}&lang=en-us&fontScale=1&preload=&baseUrl=https%3A%2F%2Fmatrix-client.matrix.org", base=BASE_URL, parent=encode(BASE_URL), user =encode(user_id), room =encode(room_id));
}
