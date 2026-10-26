use crate::community::south::port::IdentityProvider;
#[derive(Clone)]
pub struct WechatIdentity;
impl IdentityProvider for WechatIdentity {
    async fn exchange(&self, code: &str) -> Result<String, ()> {
        let appid = std::env::var("CROWD_WECHAT_APP_ID").map_err(|_| ())?;
        let secret = std::env::var("CROWD_WECHAT_APP_SECRET").map_err(|_| ())?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|_| ())?;
        let value: serde_json::Value = client
            .get("https://api.weixin.qq.com/sns/jscode2session")
            .query(&[
                ("appid", appid.as_str()),
                ("secret", secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|_| ())?
            .error_for_status()
            .map_err(|_| ())?
            .json()
            .await
            .map_err(|_| ())?;
        if value
            .get("errcode")
            .and_then(|v| v.as_i64())
            .is_some_and(|v| v != 0)
        {
            return Err(());
        }
        let openid = value["openid"]
            .as_str()
            .filter(|v| !v.is_empty())
            .ok_or(())?;
        Ok(format!("{appid}:{openid}"))
    }
}
