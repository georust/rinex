#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::header::Header;
    use rinex::version::Version;
    use rinex::hatanaka::Hatanaka;
    use rinex::observation::Crinex;
    use rinex::observation::HeaderFields;
    use rinex::constellation::Constellation;
    use std::collections::HashMap;
    #[test]
    fn test_rnx_decompression() {
        // object
        let mut decompressor = Hatanaka::new(8);
        // fake header
        let mut header = Header::basic_obs()
            .with_version(Version {
                major: 2,
                minor: 11,
            });
        header.obs = Some(observation::HeaderFields {
            crinex: Some(
                Crinex {
                    version: Version { 
                        minor: 0,
                        major: 1,
                    },
                    prog: String::from("Test"),
                    date: chrono::Utc::now().naive_utc(),
                }
            ),
            codes: {
                let mut codes: HashMap<Constellation, Vec<String>> = HashMap::new();
                codes.insert(
                    Constellation::GPS,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes.insert(
                    Constellation::Glonass,
                    vec![
                        String::from("L1"),
                        String::from("L2"),
                        String::from("C1"),
                        String::from("P2"),
                        String::from("P1"),
                        String::from("S1"),
                        String::from("S2")
                    ]);
                codes
            },
            clock_offset_applied: false,
        });
        
        // Test faulty CRX first epoch
        let content = "21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_err(), true);

        // real CRX data [epoch #1]
        let content = "&21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, ""); // Too early to output anything

        // empty clock offset [epoch #1]
        let content = "\n";
        let decompressed = decompressor.decompress(&header, content); 
        //assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15\n");
       
        // epoch#1 
        let content = "3&126298057858 3&98414080647 3&24033720416 3&24033721351 3&24033719353 3&40000 3&22000  643        4\n";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126298057.858 6  98414080.64743  24033720.416    24033721.351    24033719.353  
        40.000          22.0004 \n");
        
        
        let content = "3&111982965979 3&87259475177 3&21309646971 3&21309649924 3&21309646771 3&48000 3&37000  846        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result," 111982965.979 8  87259475.17746  21309646.971    21309649.924    21309646.771  
        48.000          37.0004 \n");
        
        let content = "3&125184221815 3&97546158899 3&23821762469 3&23821768749 3&23821762040 3&40000 3&35000  645        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 125184221.815 6  97546158.89945  23821762.469    23821768.749    23821762.040  
        40.000          35.0004 \n");
        
        let content = "3&111582858305 3&86947705907 3&21233509912 3&21233511727 3&21233508599 3&49000 3&37000  846        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111582858.305 8  86947705.90746  21233509.912    21233511.727    21233508.599  
        49.000          37.0004 \n");
        
        let content = "3&123948324318 3&96583131804 3&23586581658 3&23586584420 3&23586580391 3&39000 3&20000  643        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123948324.318 6  96583131.80443  23586581.658    23586584.420    23586580.391  
        39.000          20.0004 \n");
        
        let content = "3&121443586995 3&94631394735 3&23109944474 3&23109947055 3&23109944139 3&40000 3&28000  644        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 121443586.995 6  94631394.73544  23109944.474    23109947.055    23109944.139  
        40.000          28.0004 \n");
        
        let content = "3&123664246260 3&96183328899 3&23125836575 3&23125839071 3&23125836244 3&41000 3&40000  6 6";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123664246.260 6  96183328.899 6  23125836.575    23125839.071    23125836.244  
        41.000          40.000  \n");
        
        let content = "3&120184930156 3&93477190884 3&22506776986 3&22506777899 3&22506777465 3&44000 3&43000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 120184930.156 7  93477190.884 7  22506776.986    22506777.899    22506777.465  
        44.000          43.000  \n");
        
        let content = "3&114160130658 3&88955964556 3&21723948105 3&21723953153 3&21723947155 3&46000 3&47000  747        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 114160130.658 7  88955964.55647  21723948.105    21723953.153    21723947.155  
        46.000          47.0004 \n");
        
        let content = "3&105271498527 3&82029766966 3&20032495677 3&20032500289 3&20032495191 3&53000 3&56000  849        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 105271498.527 8  82029766.96649  20032495.677    20032500.289    20032495.191  
        53.000          56.0004 \n");
        
        let content = "3&112144051840 3&87384999714 3&21340302567 3&21340307619 3&21340301864 3&52000 3&52000  848        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 112144051.840 8  87384999.71448  21340302.567    21340307.619    21340301.864  
        52.000          52.0004 \n");

        let content = "3&113556669792 3&88485725931 3&21609114743 3&21609116936 3&21609113593 3&47000 3&38000  746        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113556669.792 7  88485725.93146  21609114.743    21609116.936    21609113.593  
        47.000          38.0004 \n");
        
        let content = "3&106844822639 3&83101546155 3&20015628375 3&20015631390 3&20015628486 3&53000 3&50000  8 8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 106844822.639 8  83101546.155 8  20015628.375    20015631.390    20015628.486  
        53.000          50.000  \n");
        
        let content = "3&131399268954 3&102389055312 3&25004448492 3&25004450593 3&25004447809 3&36000 3&12000  642        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 131399268.954 6 102389055.31242  25004448.492    25004450.593    25004447.809  
        36.000          12.0004 \n");

        let content = "3&116954721816 3&90964809026 3&21878794287 3&21878798089 3&21878794473 3&42000 3&39000  7 6";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 116954721.816 7  90964809.026 6  21878794.287    21878798.089    21878794.473  
        42.000          39.000  \n");

        let content = "3&110150054052 3&85672270182 3&20620317569 3&20620319227 3&20620317319 3&46000 3&42000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 110150054.052 7  85672270.182 7  20620317.569    20620319.227    20620317.319  
        46.000          42.000  \n");

        let content = "3&104344364269 3&81156754225 3&19499235584 3&19499235915 3&19499235267 3&52000 3&51000  8 8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 104344364.269 8  81156754.225 8  19499235.584    19499235.915    19499235.267  
        52.000          51.000  \n");
        
        let content = "3&126812563577 3&98815006750 3&24131624962 3&24131627813 3&24131624907 3&38000 3&29000  644        4";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126812563.577 6  98815006.75044  24131624.962    24131627.813    24131624.907  
        38.000          29.0004 \n");
        
        let content = "3&113907865790 3&88595032353 3&21346285819 3&21346288923 3&21346285443 3&49000 3&46000  8 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113907865.790 8  88595032.353 7  21346285.819    21346288.923    21346285.443  
        49.000          46.000  \n");

        let content = "3&118516772306 3&92179732837 3&22178802374 3&22178804901 3&22178802684 3&45000 3&42000  7 7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 118516772.306 7  92179732.837 7  22178802.374    22178804.901    22178802.684  
        45.000          42.000  \n");
/*
        let content = "                3";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result,
        
        let content = "";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result,
        
        let content = "-15603288 -12158423 -2969836 -2968829 -2968864 -1000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result,
*/
    }
}
