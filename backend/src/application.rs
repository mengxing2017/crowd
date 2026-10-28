use crate::catalog::north::local::CollectTouch;
use crate::catalog::north::local::ReadProductImage;
use crate::catalog::north::local::ReadPublicCatalog;
use crate::catalog::north::local::ReconcileCatalog;
use crate::catalog::north::local::RecordVerifiedCompletion;
use crate::files::north::local::ReadPrivateFile;
use crate::files::north::local::UploadFile;
use crate::governance::north::local::ReadAgreements;
use crate::identity::north::local::RedeemSession;
use crate::identity::north::local::StaffLogin;
use crate::identity::north::local::WechatLogin;
use crate::platform::north::local::ExecuteCommand;
use crate::platform::north::local::ReadResource;
use crate::platform::south::adapter::postgres::Repository;
use crate::promotion::north::local::ResolvePromotion;
use crate::tasks::north::local::ReadEvidenceFile;
use crate::tasks::north::local::RecordVerifiedTaskResult;

/// Composition only: each field owns one application use case.
#[derive(Clone)]
pub struct ApplicationServices {
    pub staff_login: StaffLogin,
    pub wechat_login: WechatLogin,
    pub agreements: ReadAgreements,
    pub promotions: ResolvePromotion,
    pub queries: ReadResource,
    pub commands: ExecuteCommand,
    pub uploads: UploadFile,
    pub private_files: ReadPrivateFile,
    pub session_redemption: RedeemSession,
    pub catalog_lifecycle: ReconcileCatalog,
    pub catalog: ReadPublicCatalog,
    pub touches: CollectTouch,
    pub product_images: ReadProductImage,
    pub verified_completions: RecordVerifiedCompletion,
    pub verified_task_results: RecordVerifiedTaskResult,
    pub evidence_files: ReadEvidenceFile,
}

impl ApplicationServices {
    pub fn new(repo: Repository) -> Self {
        Self {
            staff_login: StaffLogin::new(repo.clone()),
            wechat_login: WechatLogin::new(repo.clone()),
            agreements: ReadAgreements::new(repo.clone()),
            promotions: ResolvePromotion::new(repo.clone()),
            queries: ReadResource::new(repo.clone()),
            commands: ExecuteCommand::new(repo.clone()),
            uploads: UploadFile::new(repo.clone()),
            private_files: ReadPrivateFile::new(repo.clone()),
            session_redemption: RedeemSession::new(repo.clone()),
            catalog_lifecycle: ReconcileCatalog::new(repo.clone()),
            catalog: ReadPublicCatalog::new(repo.clone()),
            touches: CollectTouch::new(repo.clone()),
            product_images: ReadProductImage::new(repo.clone()),
            verified_completions: RecordVerifiedCompletion::new(repo.clone()),
            verified_task_results: RecordVerifiedTaskResult::new(repo.clone()),
            evidence_files: ReadEvidenceFile::new(repo.clone()),
        }
    }
}
