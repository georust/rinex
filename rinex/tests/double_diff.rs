/*#[cfg(test)]
mod test {
    fn test_ddiff() {
        let pool = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let obs_path = pool.to_owned() 
            + "CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz";
        let nav_path = pool.to_owned() 
            + "NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz";
        let context1 = Context1D::new(obs_path, nav_path);
        assert_eq!(context1.is_ok(), true);
        let obs_path = pool.to_owned()
            + "CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz";
        let nav_path = pool.to_owned()
            + "NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz";
        let context2 = Context1D::new(obs_path, nav_path);
        assert_eq!(context2.is_ok(), true);
        let context = DiffContext::new(context1, context2)
            .unwrap(); // already tested
        let mut context = 
            differential::Context2D::from_files(&obs_path, &nav_path);
        assert_eq!(context.is_ok(), true);
        let context = context.unwrap();
        let rhs = Rinex::from_file(&rhs_path);
        assert_eq!(rhs.is_ok(), true);
        let rhs = rhs.unwrap();
        // process
        let new_ctx = context.ddiff(rhs.clone());
        assert_eq!(new_ctx.is_ok(), true);
        let new_ctx = new_ctx.unwrap();
        // run testbench
        for (epoch, (_, vehicules)) in new_ctx.obs.record.as_obs().unwrap() {
            // sampling strictly preserved
            assert_eq!(context.obs.epochs().contains(epoch), true);
            assert_eq!(rhs.epochs().contains(epoch), true);
            // browse data
            for (vehicule, observables) in vehicules.iter() {
                for (observable, observation) in observables.iter() {
                    if is_carrier_phase_obs_code!(observable) {
                        // if this is vehicule was not selected as ref. vehicule
                        // compute tb
                    } else {
                        // non phase data:
                        // strictly preserved
                        assert_eq!(
                    }
                }
            }
        }
    }
}*/
