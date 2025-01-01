function hideContentByDocumentId(id) {
    var content = document.getElementById(id);
    if (content != null) {
        content.style.display = 'none';
    }
}

function showContentByDocumentId(id) {
    var content = document.getElementById(id);
    if (content != null) {
        content.style.display = 'block';
    }
}

function hideShowContentByDocumentId(sectionId) {
    var content = document.getElementById(sectionId);
    if (content.display == 'none' || content.display == "") {
        content.style.display = "block";
    } else {
        content.style.display = "none";
    }
}

function toggleElementVisibilityByDocumentId(id) {
    console.log("toggling " + id);
    var content = document.getElementById(id);
    if (content.style.display == 'none' || content.style.display == "") {
        content.style.display = "block";
    } else {
        content.style.display = "none";
    }
}

function getHeroContentById(id) {
    var heros = document.getElementsByClassName("section");
    for (var hero of heros) {
        console.log("hero: " + hero.id);
        if (hero.id == id) {
            return hero;
        }
    }
    return null;
}

function getQcSummaryHero() {
    return getHeroContentById("qc-summary");
}

function getQcObservationsHero() {
    return getHeroContentById("qc-observations");
}

function showQcSummary() {
    console.log("show: qc-summary");
    let hero = getQcSummaryHero();
    hero.style.display = 'block';
}

function hideQcSummary() {
    console.log("hide: qc-summary");
    let hero = getQcSummaryHero();
    hero.style.display = 'none';
}


function showQcObservations() {
    var hero = getQcSummaryHero();
    hero.display.style = 'none';

    var hero = getQcObservationsHero();
    if (hero != null) {
        hero.display.style = 'block';
    }
}

function onQcSummaryClicks() {
    var hero = getQcSummaryHero();
    hero.style = 'block';

    var hero = getQcObservationsHero();
    if (hero != null) {
        hero.style = 'none';
    }
}

function onQcObservationsClicks() {
    var hero = getQcSummaryHero();
    hero.style = 'none';

    var hero = getQcObservationsHero();
    if (hero != null) {
        hero.style = 'block';
    }
}

// function showQcSelectedGnssRx(rx) {
//     console.log("showing: " +rx);
//     var gnss_receivers = document.getElementsByClassName("qc-obs-receiver");
//     for (var gnss_rx of gnss_receivers) {
//         if (gnss_rx.id == rx) {
//             gnss_rx.style.display = 'block';
//         } else {
//             gnss_rx.style.display = 'none';
//         }
//     }
// }

function onQcNaviSummaryConstellationChanges(changes) {
    console.log("YES changes: " + changes.target.value);
}

function buildPageListeners() {
    // Summary listeners
    // 1. qc-navi-sum for each rover, for each constellation
    let navi_summary_selector = document.getElementsByClassName("qc-navi-sum-selector");

    for (selector of navi_summary_selector) {
        selector.onchange = function(changes) {
            var constellation = changes.target.value;
            console.log("qc-navi-sum constellation: " + constellation);
            
            var selected = document.getElementsByClassName("qc-navi-sum-selected");
            for (fields of selected) {
                var id = fields.id;
                if (id == constellation) {
                    fields.style.display = "block";
                } else {
                    fields.style.display = "none";
                }
            }
        }
    }
    
    // qc-sidemenu listeners
    let qc_side_menu = document.getElementsByClassName("qc-sidemenu");
    for (item of qc_side_menu) {
        var item_id = item.id;
        console.log("qc-sidemenu item: " + item_id);

        if (item_id == "qc-observations") {
            item.e
            item.onchange = function(changes) {
                console.log(changes.target.value);
            };
        }
    }
}