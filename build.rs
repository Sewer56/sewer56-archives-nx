fn main() {
    // Build time scripts go here. If you have nothing to do here, you can remove this file.
csbindgen::Builder::default()
        .input_extern_file("src/exports.rs")
        .csharp_dll_name("sewer56_archives_nx")
        .csharp_class_accessibility("public")
        .csharp_namespace("sewer56_archives_nx.Net.Sys")
        .generate_csharp_file("bindings/csharp/NativeMethods.g.cs")
        .unwrap();

}
