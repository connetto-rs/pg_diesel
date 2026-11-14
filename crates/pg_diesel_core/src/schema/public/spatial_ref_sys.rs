//! Submodule for the `public.spatial_ref_sys` table schema.

diesel::table! {
    /// `public.spatial_ref_sys` — `PostGIS` table containing spatial reference systems.
    /// This table is standard in `PostGIS` columns but installed in public by the test setup.
    public.spatial_ref_sys (srid) {
        /// The Spatial Reference System Identifier (SRID).
        srid -> Integer,
        /// The authority name (e.g., 'EPSG').
        auth_name -> Nullable<VarChar>,
        /// The authority's SRID (e.g., 4326).
        auth_srid -> Nullable<Integer>,
        /// The WKT representation of the SRS.
        srtext -> Nullable<VarChar>,
        /// The PROJ4 representation of the SRS.
        proj4text -> Nullable<VarChar>,
    }
}
