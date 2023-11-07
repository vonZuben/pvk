use crate::simple_parse::TokenIter;

/// some api are exposed only for vulkan, only for vulkansc, or both
/// we currently only support api which is for vulkan
pub(crate) fn api_for_vulkan(api: &str) -> bool {
    let tokens = TokenIter::new(api);

    for token in tokens {
        if token == "vulkan" {
            return true;
        } else if token == "disabled" {
            return false;
        }
    }

    false
}
