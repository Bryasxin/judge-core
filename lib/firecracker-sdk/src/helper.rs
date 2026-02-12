/// Used for quickly generating builder pattern setter methods
/// HACK: This is a temporary method and will be modified later.
#[macro_export]
macro_rules! with {
    // Match [`Option<T>`]
    ($field_name:expr, Option<$inner_type:ty>) => {
        paste::paste! {
            pub fn [<with_ $field_name>](&mut self, $field_name: $inner_type) -> &mut Self {
                self.[<$field_name>] = Some($field_name);
                self
            }
        }
    };
    ($method_name:expr, $field_name:expr, Option<$inner_type:ty>) => {
        paste::paste! {
            pub fn [<with_ $method_name>](&mut self, $field_name: $inner_type) -> &mut Self {
                self.[<$field_name>] = Some($field_name);
                self
            }
        }
    };

    // Match normal types
    ($field_name:expr, $field_type:ty) => {
        paste::paste! {
            pub fn [<with_ $field_name>](&mut self, $field_name: $field_type) -> &mut Self {
                self.[<$field_name>] = $field_name;
                self
            }
        }
    };
    ($method_name:expr, $field_name:expr, $field_type:ty) => {
        paste::paste! {
            pub fn [<with_ $method_name>](&mut self, $field_name: $field_type) -> &mut Self {
                self.[<$field_name>] = $field_name;
                self
            }
        }
    };
}
