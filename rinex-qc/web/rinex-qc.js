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
    return getHeroContentById("qc-obs");
}

function showQcSummary() {
    console.log("show: qc-summary");
    let hero = getQcSummaryHero();
    hero.style.display = 'block';
    hideQcObservations();
}

function hideQcSummary() {
    console.log("hide: qc-summary");
    let hero = getQcSummaryHero();
    hero.style.display = 'none';
}

function showQcObservations() {
    console.log("show: qc-obs");

    // unlock this section and its summary
    showContentByDocumentId("qc-obs");
    showContentByDocumentId("qc-obs-summary");

    // hide other sections
    hideQcSummary();
}

function hideQcObservations() {
    console.log("show: qc-obs");
    showContentByDocumentId("qc-obs");
}

function showQcSelectedGnssRx(rx) {
    console.log("showing: " +rx);
    var gnss_receivers = document.getElementsByClassName("qc-obs-receiver");
    for (var gnss_rx of gnss_receivers) {
        if (gnss_rx.id == rx) {
            gnss_rx.style.display = 'block';
        } else {
            gnss_rx.style.display = 'none';
        }
    }
}

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
    
    // builds page dependent listeners
    // 1. obs-receivers content display (if any)
    var receivers = document.getElementById("qc-obs-receivers");
    if (receivers != null) {
        receivers.onchange = function(changes) {
            var selected_gnss_rx = changes.target.value;
            console.log("selected receiver: " + selected_gnss_rx);
            showQcSelectedGnssRx(selected_gnss_rx);
        }
    }

    // for each qc-obs-receivers: create the constellation pagination listener
    // for each qc-obs-constellations: create the signal pagination listener;
}