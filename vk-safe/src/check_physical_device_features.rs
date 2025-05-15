/// Make a type that can be used to query Vulkan feature support
///
/// Provide a list of the feature structs with features that you want to check.
/// The resulting type can be used in the Pnext chain of
/// [get_physical_device_features2](crate::vk::PhysicalDevice::get_physical_device_features2)
/// in order to check if the features are supported.
///
/// You should only indicate the extension features you want to check. The default
/// features provided by [PhysicalDeviceFeatures](crate::vk::PhysicalDeviceFeatures2) are
/// obtained by default. If you only want to obtain the default features, then
/// just use [get_physical_device_features](crate::vk::PhysicalDevice::get_physical_device_features).
#[macro_export]
macro_rules! check_physical_device_features {
    ( $($name:ident),+ $(,)? ) => {{
        $crate::p_next_inner!( CheckFeatures, Pnext : $($name),* );

        impl<Tag> $crate::raw::CheckPhysicalDeviceFeatures for CheckFeatures<$crate::vk::PhysicalDeviceFeatures2<Tag>, Tag> {
            $(
                $crate::raw::$name!( $name );
            )*
        }

        Pnext
    }};
}
