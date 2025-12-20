pub mod argondb_service_definition {
    tonic::include_proto!("argondb");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("argondb_descriptor");
}
