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
    fn test_crx1_decompression() {
        // object
        let mut decompressor = Hatanaka::new(8)
            .unwrap();
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

        // from now on, we browse a valid CRNX
        // and compare each uncompressed (dumped) data to 
        // data that was manually uncompressed with "CRX2RNX"

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
        
        // epoch #2
        let content = "                3";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "\n";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 21  1  1  0  0 30.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15\n");
       
        //////////////////////////////////////
        // epoch #2 data #1
        // first partially compressed data
        //////////////////////////////////////
        let content = "-15603288 -12158423 -2969836 -2968829 -2968864 -1000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126282454.570 6  98401922.22443  24030750.580    24030752.522    24030750.489  
        39.000          22.0004 \n");
    
        let content = "-16266911 -12675508 -3095428 -3095463 -3095468 -2000 1000  7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111966699.068 7  87246799.66946  21306551.543    21306554.461    21306551.303  
        46.000          38.0004 \n");

        let content = "76851633 59884369 14624438 14624373 14624295 -2000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 125261073.448 6  97606043.26845  23836386.907    23836393.122    23836386.335  
        38.000          35.0004 \n");
        
        let content = "-6132968 -4778936 -1167231 -1166970 -1167080 -1000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111576725.337 8  86942926.97146  21232342.681    21232344.757    21232341.519  
        48.000          37.0004 \n");
        
        let content = "-120758522 -94097544 -22980041 -22980287 -22979836 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123827565.796 6  96489034.26043  23563601.617    23563604.133    23563600.555  
        39.000          20.0004 \n");
        
        let content = "62033196 48337565 11804471 11804740 11804239 2000 0  7";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 121505620.191 7  94679732.30044  23121748.945    23121751.795    23121748.378  
        42.000          28.0004 \n");
        
        let content = "69391635 53971217 12976205 12976871 12977235 0 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123733637.895 6  96237300.116 6  23138812.780    23138815.942    23138813.479  
        41.000          39.000  \n");
        
        let content = "-122441918 -95232607 -22928378 -22929106 -22929666 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 120062488.238 7  93381958.277 7  22483848.608    22483848.793    22483847.799  
        44.000          43.000  \n");
        
        let content = "-109755506 -85523763 -20885942 -20885839 -20885758 1000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 114050375.152 7  88870440.79347  21703062.163    21703067.314    21703061.397  
        47.000          47.0004 \n");
        
        let content = "-42812752 -33360601 -8146661 -8146987 -8146980 0 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 105228685.775 8  81996406.36549  20024349.016    20024353.302    20024348.211  
        53.000          57.0004 \n");
        
        let content = "-83350333 -64948295 -15861167 -15861192 -15861042 -2000 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 112060701.507 8  87320051.41948  21324441.400    21324446.427    21324440.822  
        50.000          53.0004 \n");
        
        let content = "48656547 37914182 9258742 9259052 9258997 1000 -1000  8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113605326.339 8  88523640.11346  21618373.485    21618375.988    21618372.590  
        48.000          37.0004 \n");
        
        let content = "-91469176 -71142692 -17134886 -17135341 -17135153 0 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 106753353.463 8  83030403.463 8  19998493.489    19998496.049    19998493.333  
        53.000          49.000  \n");
        
        let content = "-12490564 -9732957 -2375281 -2377415 -2376889 1000 -1000    1";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 131386778.390 6 102379322.35541  25002073.211    25002073.178    25002070.920  
        37.000          11.0004 \n");
        
        let content = "17043835 13256315 3189446 3188124 3188690 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 116971765.651 7  90978065.341 6  21881983.733    21881986.213    21881983.163  
        42.000          39.000  \n");
        
        let content = "-9092529 -7071983 -1702369 -1702302 -1702186 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 110140961.523 7  85665198.199 7  20618615.200    20618616.925    20618615.133  
        46.000          42.000  \n");
        
        let content = "6325082 4919499 1182061 1182031 1182111 0 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 104350689.351 8  81161673.724 8  19500417.645    19500417.946    19500417.378  
        52.000          50.000  \n");
        
        let content = "-36190639 -28200493 -6886905 -6886517 -6887081 -2000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126776372.938 6  98786806.25744  24124738.057    24124741.296    24124737.826  
        36.000          29.0004 \n");
        
        let content = "-123519150 -96070428 -23148024 -23147650 -23147459 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113784346.640 8  88498961.925 7  21323137.795    21323141.273    21323137.984  
        49.000          46.000  \n");
        
        let content = "89879429 69906214 16820349 16819910 16819607 1000 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 118606651.735 7  92249639.051 7  22195622.723    22195624.811    22195622.291  
        46.000          43.000  \n");

        //////////////////////////////////////
        // epoch#3 data gets more compressed
        //////////////////////////////////////
        let content = "              1 &";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        
        let content = "\n";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 21  1  1  0  1  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15\n");

        let content = "521089 406062 100164 98225 98271 2000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126267372.371 6  98390169.86343  24027880.908    24027881.918    24027879.896  
        40.000          22.0004 \n");
        
        let content = "609858 475212 116242 116012 116115 4000 -2000  8";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111951042.015 8  87234599.37346  21303572.357    21303575.010    21303571.950  
        48.000          37.0004 \n");
        
        let content = "111109 86584 21163 21212 21363 3000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 125338036.190 6  97666014.22145  23851032.508    23851038.707    23851031.993  
        39.000          35.0004 \n");
        
        let content = "566742 441617 107872 107743 107850 2000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111571159.111 8  86938589.65246  21231283.322    21231285.530    21231282.289  
        49.000          37.0004 \n");
        
        let content = "189405 147603 37458 37895 36854 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123706996.679 6  96395084.31943  23540659.034    23540661.741    23540657.573  
        39.000          20.0004 \n");
        
        let content = "136034 105988 25870 25574 26379 -3000 -1000  6";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 121567789.421 6  94728175.85344  23133579.286    23133582.109    23133578.996  
        41.000          27.0004 \n");
        
        let content = "66853 51998 12691 12273 11789 0 2000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 123803096.383 6  96291323.331 6  23151801.676    23151805.086    23151802.503  
        41.000          40.000  \n");
        
        let content = "584785 454803 108985 108662 110848 0 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 119940631.105 7  93287180.473 7  22461029.215    22461028.349    22461028.981  
        44.000          42.000  \n");
        
        let content = "258954 201780 49125 49244 49311 -2000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113940878.600 7  88785118.81047  21682225.346    21682230.719    21682224.950  
        46.000          47.0004 \n");
        
        let content = "447309 348555 84693 85091 85079 0 -2000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 105186320.332 8  81963394.31949  20016287.048    20016291.406    20016286.310  
        53.000          56.0004 \n");
        
        let content = "528629 411917 100672 100687 100548 3000 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 111977879.803 8  87255515.04148  21308680.905    21308685.922    21308680.328  
        51.000          53.0004 \n");
        
        let content = "409272 318921 78128 77864 77853 -1000 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113654392.158 8  88561873.21646  21627710.355    21627712.904    21627709.440  
        48.000          37.0004 \n");
        
        let content = "356412 277209 66153 66924 66484 0 2000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 106662240.699 8  82959537.980 8  19981424.756    19981427.632    19981424.664  
        53.000          50.000  \n");
        
        let content = "565640 440810 105412 109222 109538 -2000 2000    2";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 131374853.466 6 102370030.20842  24999803.342    24999804.985    24999803.569  
        36.000          12.0004 \n");
        
        let content = "748321 582041 138218 140738 139637 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 116989557.807 7  90991903.697 6  21885311.397    21885315.075    21885311.490  
        42.000          39.000  \n");
        
        let content = "815590 634344 153316 152881 152832 0 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 110132684.584 7  85658760.560 7  20617066.147    20617067.504    20617065.779  
        46.000          43.000  \n");
        
        let content = "384203 298827 71632 71657 71623 1000 1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 104357398.636 8  81166892.050 8  19501671.338    19501671.634    19501671.112  
        53.000          50.000  \n");
        
        let content = "599215 466931 114210 113430 114161 4000 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 126740781.514 6  98759072.69544  24117965.362    24117968.209    24117964.906  
        38.000          29.0004 \n");
        
        let content = "576174 448136 108266 108219 108061 0 0";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 113661403.664 8  88403339.633 7  21300098.037    21300101.842    21300098.586  
        49.000          46.000  \n");
        
        let content = "284828 221531 51578 52840 53501 -1000 -1000";
        let decompressed = decompressor.decompress(&header, content); 
        assert_eq!(decompressed.is_ok(), true);
        let result = decompressed.unwrap();
        assert_eq!(result, " 118696815.992 7  92319766.796 7  22212494.650    22212497.561    22212495.399  
        46.000          43.000  \n");
    }
}
