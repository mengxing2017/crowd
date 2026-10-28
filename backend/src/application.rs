use crate::catalog::south::port::services::CollectTouch;
use crate::catalog::south::port::services::ReadProductImage;
use crate::catalog::south::port::services::ReadPublicCatalog;
use crate::files::south::port::services::ReadPrivateFile;
use crate::files::south::port::services::UploadFile;
use crate::governance::south::port::services::ReadAgreements;
use crate::identity::south::port::services::RedeemSession;
use crate::identity::south::port::services::StaffLogin;
use crate::identity::south::port::services::WechatLogin;
use crate::platform::south::port::Repository;
use crate::platform::south::port::services::ExecuteCommand;
use crate::platform::south::port::services::ReadResource;
use crate::promotion::south::port::services::ResolvePromotion;

/// Composition only: each field owns one application use case.
#[derive(Clone)]
pub struct ApplicationServices {
    pub staff_login: StaffLogin,
    pub wechat_login: WechatLogin,
    pub agreements: ReadAgreements,
    pub promotions: ResolvePromotion,
    pub queries: ReadResource,
    pub withdrawal_failure_context:
        crate::withdrawals::south::port::services::ReadWithdrawalFailureContext,
    pub commands: ExecuteCommand,
    pub uploads: UploadFile,
    pub private_files: ReadPrivateFile,
    pub session_redemption: RedeemSession,
    pub catalog: ReadPublicCatalog,
    pub touches: CollectTouch,
    pub product_images: ReadProductImage,
    pub sensitive_files: crate::files::south::port::services::ReadSensitiveFile,
}

impl ApplicationServices {
    pub fn new(repo: Repository) -> Self {
        Self {
            staff_login: StaffLogin::new(repo.clone()),
            wechat_login: WechatLogin::new(repo.clone()),
            agreements: ReadAgreements::new(repo.clone()),
            promotions: ResolvePromotion::new(repo.clone()),
            queries: ReadResource::new(repo.clone()),
            withdrawal_failure_context:
                crate::withdrawals::south::port::services::ReadWithdrawalFailureContext::new(
                    repo.clone(),
                ),
            commands: ExecuteCommand::new(repo.clone()),
            uploads: UploadFile::new(repo.clone()),
            private_files: ReadPrivateFile::new(repo.clone()),
            session_redemption: RedeemSession::new(repo.clone()),
            catalog: ReadPublicCatalog::new(repo.clone()),
            touches: CollectTouch::new(repo.clone()),
            product_images: ReadProductImage::new(repo.clone()),
            sensitive_files: crate::files::south::port::services::ReadSensitiveFile::new(
                repo.clone(),
            ),
        }
    }
}
