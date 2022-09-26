#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::header::Header;
    use rinex::version::Version;
    use rinex::hatanaka::Compressor;
    use rinex::observation;
    use rinex::observation::Crinex;
    use std::collections::HashMap;
    use rinex::constellation::Constellation;
    #[test]
    fn crx1_compression() {
        // object
        let mut compressor = Compressor::new();
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
        
        // RNX epoch
        let content = " 21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // + empty clock offset line, due to missing field [epoch #1]
        let expected = "&21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15\n\n";
        assert_eq!(result, expected);
        
        // epoch#1 sat#1
        let content = " 126298057.858 6  98414080.64743  24033720.416    24033721.351    24033719.353";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&126298057858 3&98414080647 3&24033720416 3&24033721351 3&24033719353 ");

        let content = "        40.000          22.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&40000 3&22000  643        4\n");

        // epoch#1 sat#2
        let content = " 111982965.979 8  87259475.17746  21309646.971    21309649.924    21309646.771";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&111982965979 3&87259475177 3&21309646971 3&21309649924 3&21309646771 ");
        let content = "        48.000          37.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&48000 3&37000  846        4\n");

        // epoch#1 sat#3
        let content = " 125184221.815 6  97546158.89945  23821762.469    23821768.749    23821762.040";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "3&125184221815 3&97546158899 3&23821762469 3&23821768749 3&23821762040 ");
        
        let content = "        40.000          35.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&40000 3&35000  645        4\n");

        // epoch#1 sat#4
        let content = " 111582858.305 8  86947705.90746  21233509.912    21233511.727    21233508.599";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&111582858305 3&86947705907 3&21233509912 3&21233511727 3&21233508599 ");
        
        let content = "        49.000          37.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&49000 3&37000  846        4\n");

        let content = " 123948324.318 6  96583131.80443  23586581.658    23586584.420    23586580.391";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&123948324318 3&96583131804 3&23586581658 3&23586584420 3&23586580391 ");
        
        let content = "        39.000          20.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&39000 3&20000  643        4\n");

        let content = " 121443586.995 6  94631394.73544  23109944.474    23109947.055    23109944.139";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&121443586995 3&94631394735 3&23109944474 3&23109947055 3&23109944139 ");
        
        let content = "        40.000          28.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&40000 3&28000  644        4\n");

        let content = " 123664246.260 6  96183328.899 6  23125836.575    23125839.071    23125836.244";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&123664246260 3&96183328899 3&23125836575 3&23125839071 3&23125836244 ");
        
        let content = "        41.000          40.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&41000 3&40000  6 6\n");

        let content = " 120184930.156 7  93477190.884 7  22506776.986    22506777.899    22506777.465";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&120184930156 3&93477190884 3&22506776986 3&22506777899 3&22506777465 ");
        
        let content = "        44.000          43.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&44000 3&43000  7 7\n");

        let content = " 114160130.658 7  88955964.55647  21723948.105    21723953.153    21723947.155";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&114160130658 3&88955964556 3&21723948105 3&21723953153 3&21723947155 ");
        
        let content = "        46.000          47.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&46000 3&47000  747        4\n");

        let content = " 105271498.527 8  82029766.96649  20032495.677    20032500.289    20032495.191";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&105271498527 3&82029766966 3&20032495677 3&20032500289 3&20032495191 ");
        
        let content = "        53.000          56.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&53000 3&56000  849        4\n");

        let content = " 112144051.840 8  87384999.71448  21340302.567    21340307.619    21340301.864";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&112144051840 3&87384999714 3&21340302567 3&21340307619 3&21340301864 ");
        
        let content = "        52.000          52.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&52000 3&52000  848        4\n");

        let content = " 113556669.792 7  88485725.93146  21609114.743    21609116.936    21609113.593";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&113556669792 3&88485725931 3&21609114743 3&21609116936 3&21609113593 ");
        
        let content = "        47.000          38.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&47000 3&38000  746        4\n");

        let content = " 106844822.639 8  83101546.155 8  20015628.375    20015631.390    20015628.486";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&106844822639 3&83101546155 3&20015628375 3&20015631390 3&20015628486 ");
        
        let content = "        53.000          50.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Completion 
        assert_eq!(result, "3&53000 3&50000  8 8\n");
        
        let content = " 131399268.954 6 102389055.31242  25004448.492    25004450.593    25004447.809";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial 
        assert_eq!(result, "3&131399268954 3&102389055312 3&25004448492 3&25004450593 3&25004447809 ");

        let content = "        36.000          12.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Completion 
        assert_eq!(result, "3&36000 3&12000  642        4\n");
        
        let content = " 116954721.816 7  90964809.026 6  21878794.287    21878798.089    21878794.473";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&116954721816 3&90964809026 3&21878794287 3&21878798089 3&21878794473 ");
        
        let content = "        42.000          39.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&42000 3&39000  7 6\n");

        let content = " 110150054.052 7  85672270.182 7  20620317.569    20620319.227    20620317.319";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&110150054052 3&85672270182 3&20620317569 3&20620319227 3&20620317319 ");
        
        let content = "        46.000          42.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&46000 3&42000  7 7\n");

        let content = " 104344364.269 8  81156754.225 8  19499235.584    19499235.915    19499235.267";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&104344364269 3&81156754225 3&19499235584 3&19499235915 3&19499235267 ");
        
        let content = "        52.000          51.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&52000 3&51000  8 8\n");

        let content = " 126812563.577 6  98815006.75044  24131624.962    24131627.813    24131624.907";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&126812563577 3&98815006750 3&24131624962 3&24131627813 3&24131624907 ");
        
        let content = "        38.000          29.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&38000 3&29000  644        4\n");

        let content = " 113907865.790 8  88595032.353 7  21346285.819    21346288.923    21346285.443";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&113907865790 3&88595032353 3&21346285819 3&21346288923 3&21346285443 ");
        
        let content = "        49.000          46.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&49000 3&46000  8 7\n");

        let content = " 118516772.306 7  92179732.837 7  22178802.374    22178804.901    22178802.684";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // Partial CRNX line
        assert_eq!(result, "3&118516772306 3&92179732837 3&22178802374 3&22178804901 3&22178802684 ");
        
        let content = "        45.000          42.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // CRNX line completion
        assert_eq!(result, "3&45000 3&42000  7 7\n");

        //epoch#2 first compressed epoch
        let content = " 21  1  1  0  0 30.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16
                                R18G13R01R16R17G15R02R15";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        // + empty clock offset line, due to missing field
        assert_eq!(result, "                3\n\n");
        
        //epoch#2 sat#1 1st compression
        let content =" 126282454.570 6  98401922.22443  24030750.580    24030752.522    24030750.489";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-15603288 -12158423 -2969836 -2968829 -2968864 ");
        
        let content ="        39.000          22.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-1000 0 \n");

        //epoch#2 sat#2 1st compression
        let content =" 111966699.068 7  87246799.66946  21306551.543    21306554.461    21306551.303";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-16266911 -12675508 -3095428 -3095463 -3095468 ");
        
        let content ="        46.000          38.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-2000 1000  7\n");
        
        //epoch#2 sat#3
        let content =" 125261073.448 6  97606043.26845  23836386.907    23836393.122    23836386.335";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "76851633 59884369 14624438 14624373 14624295 ");
        
        let content ="        38.000          35.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-2000 0 \n");
        
        let content =" 111576725.337 8  86942926.97146  21232342.681    21232344.757    21232341.519";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-6132968 -4778936 -1167231 -1166970 -1167080 ");
        
        let content ="        48.000          37.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-1000 0 \n");
        
        let content =" 123827565.796 6  96489034.26043  23563601.617    23563604.133    23563600.555";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-120758522 -94097544 -22980041 -22980287 -22979836 ");
        
        let content ="        39.000          20.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "0 0 \n");
        
        let content =" 121505620.191 7  94679732.30044  23121748.945    23121751.795    23121748.378";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "62033196 48337565 11804471 11804740 11804239 ");
        
        let content ="        42.000          28.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "2000 0  7\n");
        
        let content =" 123733637.895 6  96237300.116 6  23138812.780    23138815.942    23138813.479";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "69391635 53971217 12976205 12976871 12977235 ");
        
        let content ="        41.000          39.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "0 -1000 \n");
        
        let content =" 120062488.238 7  93381958.277 7  22483848.608    22483848.793    22483847.799";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-122441918 -95232607 -22928378 -22929106 -22929666 ");
        
        let content ="        44.000          43.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "0 0 \n");
        
        let content =" 114050375.152 7  88870440.79347  21703062.163    21703067.314    21703061.397";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-109755506 -85523763 -20885942 -20885839 -20885758 ");
        
        let content ="        47.000          47.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "1000 0 \n");
        
        let content =" 105228685.775 8  81996406.36549  20024349.016    20024353.302    20024348.211";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-42812752 -33360601 -8146661 -8146987 -8146980 ");
        
        let content ="        53.000          57.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "0 1000 \n");
        
        let content =" 112060701.507 8  87320051.41948  21324441.400    21324446.427    21324440.822";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-83350333 -64948295 -15861167 -15861192 -15861042 ");
        
        let content ="        50.000          53.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-2000 1000 \n");
        
        let content =" 113605326.339 8  88523640.11346  21618373.485    21618375.988    21618372.590";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "48656547 37914182 9258742 9259052 9258997 ");
        
        let content ="        48.000          37.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "1000 -1000  8\n");
        
        let content =" 106753353.463 8  83030403.463 8  19998493.489    19998496.049    19998493.333";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-91469176 -71142692 -17134886 -17135341 -17135153 ");
        
        let content ="        53.000          49.000";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "0 -1000 \n");
        
        let content =" 131386778.390 6 102379322.35541  25002073.211    25002073.178    25002070.920";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "-12490564 -9732957 -2375281 -2377415 -2376889 ");
        
        let content ="        37.000          11.0004";
        let compressed = compressor.compress(&header, content); 
        assert_eq!(compressed.is_ok(), true);
        let result = compressed.unwrap();
        assert_eq!(result, "1000 -1000    1\n");
/*        
        let content =" 116971765.651 7  90978065.341 6  21881983.733    21881986.213    21881983.163";
        assert_eq!(result, "17043835 13256315 3189446 3188124 3188690");
        let content ="        42.000          39.000";
        assert_eq!(result, "0 0");
        let content =" 110140961.523 7  85665198.199 7  20618615.200    20618616.925    20618615.133";
        assert_eq!(result, "-9092529 -7071983 -1702369 -1702302 -1702186");
        let content ="        46.000          42.000";
        assert_eq!(result, "0 0");
        let content =" 104350689.351 8  81161673.724 8  19500417.645    19500417.946    19500417.378";
        assert_eq!(result, "6325082 4919499 1182061 1182031 1182111");
        let content ="        52.000          50.000";
        assert_eq!(result, "0 -1000");
        let content =" 126776372.938 6  98786806.25744  24124738.057    24124741.296    24124737.826";
        assert_eq!(result, "-36190639 -28200493 -6886905 -6886517 -6887081");
        let content ="        36.000          29.0004";
        assert_eq!(result, "-2000 0");
        let content =" 113784346.640 8  88498961.925 7  21323137.795    21323141.273    21323137.984";
        assert_eq!(result, "-123519150 -96070428 -23148024 -23147650 -23147459");
        let content ="        49.000          46.000";
        assert_eq!(result, "0 0");
        let content =" 118606651.735 7  92249639.051 7  22195622.723    22195624.811    22195622.291";
        assert_eq!(result, "89879429 69906214 16820349 16819910 16819607");
        let content ="        46.000          43.000";
        assert_eq!(result, "1000 1000");

        //TODO
        // pass next epoch description in two invokations
*/    
    }
}
