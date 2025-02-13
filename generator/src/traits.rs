use krs_quote::{krs_quote_with, ToTokens};

pub struct VulkanCommand;

impl ToTokens for VulkanCommand {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait VulkanCommand : Copy + Sized {
                const VK_NAME: *const c_char;
                unsafe fn new(ptr: PFN_vkVoidFunction) -> Self;
            }

            /// local type alias for vulkan void function
            pub type VkVoidFunction = PFN_vkVoidFunction;

            /// "trait alias" for a function that can load a vulkan command
            pub trait FunctionLoader:
                Fn(*const c_char) -> Option<VkVoidFunction> + Copy
            {
            }
            impl<F> FunctionLoader for F where
                F: Fn(*const c_char) -> Option<VkVoidFunction> + Copy
            {
            }

            /// Error loading a command
            ///
            /// ## Safety
            /// 'command' must be set to a valid c string pointer
            /// there is no check for this
            pub struct CommandLoadError {
                command: *const c_char,
            }

            impl std::error::Error for CommandLoadError {}

            impl std::fmt::Display for CommandLoadError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    // SAFETY : CommandLoadError can only be internally created, and we must ensure it is created with a pointer to a valid c string
                    let command_name = unsafe { std::ffi::CStr::from_ptr(self.command) };
                    write!(f, "failed to load {:?}", command_name)
                }
            }

            impl std::fmt::Debug for CommandLoadError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(self, f)
                }
            }

            /// Load commands with a given function loader
            ///
            /// 'loader' is an function that takes a c_string pointer to the name of the command to load
            pub trait LoadCommands : Sized {
                fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError>;
            }

            impl<C: VulkanCommand> LoadCommands for C {
                fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError> {
                    let fptr = loader(C::VK_NAME).ok_or(CommandLoadError { command: C::VK_NAME })?;
                    // SAFETY : fptr should be the correct kind of pointer since we loaded it with H::VK_NAME
                    unsafe { Ok(C::new(fptr)) }
                }
            }

            pub trait Version {
                const VERSION: crate::VkVersion;
            }

            pub unsafe trait Stype {
                const S_TYPE: StructureType;
            }

            /// Indicates that a pointer to Self can be safely read and written to as a pointer to BaseInStructure
            ///
            /// also provides some maybe useful convenience methods
            pub unsafe trait BaseStructure: Stype {
                fn p_next(&self) -> *const BaseInStructure;
                fn as_base_structure(&self) -> *const BaseInStructure;
                unsafe fn set_p_next(&mut self, p_next: *const BaseInStructure);
            }

            /// Indicates that a pointer to Self can be safely read and written to as a pointer to BaseOutStructure
            ///
            /// also provides some maybe useful convenience methods
            pub unsafe trait BaseStructureMut: Stype {
                fn p_next_mut(&mut self) -> *mut BaseOutStructure;
                fn as_base_structure_mut(&mut self) -> *mut BaseOutStructure;
                unsafe fn set_p_next_mut(&mut self, p_next: *mut BaseOutStructure);
            }

            /// The implementor can be added to the pNext chain of `T`
            pub unsafe trait StructExtends<T> {}
        )
    }
}
