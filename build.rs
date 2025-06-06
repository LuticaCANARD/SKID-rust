fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/model/skid_color.rs")
        .input_extern_file("src/model/skid_image.rs")
        .input_extern_file("src/model/skid_vector2.rs")
        .input_extern_file("src/model/skid_vector3.rs")
        .input_extern_file("src/model/ffi_modules/skid_image_ffi.rs")
        .input_extern_file("src/model/ffi_modules/mod.rs")
        .input_extern_file("src/api/ffi_modules/mod.rs")
        .csharp_dll_name("skid_rust_backend")
        .csharp_namespace("LuticaSKIDBinder")
        .csharp_class_name("LuticaSKIDBinderToCSharp")
        .generate_csharp_file("dotnet/LuticaSKIDBinderToCSharp.g.cs")
        .unwrap();
}