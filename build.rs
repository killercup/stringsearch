fn main() {
    // TODO: Only compile+link C library in test

    cc::Build::new()
        .flag("-DHAVE_CONFIG_H")
        .flag("-DNDEBUG")
        .flag("/O2")
        .flag("/w") // I know, I know
        .file("original/divsufsort.c")
        .file("original/sssort.c")
        .file("original/trsort.c")
        .file("original/utils.c")
        .compile("libdivsufsort.a");
}
