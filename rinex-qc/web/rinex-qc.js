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

function toggleVisiblityByDocumentId(sectionId) {
    var content = document.getElementById(sectionId);
    if (content.display == 'none' || content.display == "") {
        content.style.display = "block";
    } else {
        content.style.display = "none";
    }
}

function getQcSummary() {
    return document.getElementById("qc-summary");
}

function getQcRoverObservations() {
    return document.getElementById("qc-rover-observations");
}

function getQcBaseStationsObservations() {
    return document.getElementById("qc-base-observations");
}

function showQcSummary() {
    let hero = getQcSummary();
    hero.style.display = 'block';
}

function hideQcSummary() {
    let hero = getQcSummary();
    hero.style.display = 'none';
}

function onQcSummaryClicks() {
    showQcSummary();

    var hero = getQcRoverObservations();
    if (hero != null) {
        hero.style = 'none';
    }

    var hero = getQcBaseStationsObservations();
    if (hero != null) {
        hero.style = 'none';
    }
}

function onQcRoverObsSelection(opts) {
    console.log("opts: " + opts);
}

function onQcRoverObservationsClicks() {
    hideQcSummary();

    var hero = getQcRoverObservations();
    if (hero != null) {
        hero.style = 'block';
    } else {
        console.log("is null");
    }

    var hero = getQcBaseStationsObservations();
    if (hero != null) {
        hero.style = 'none';
    } 
}

function onQcBaseObservationsClicks() {
    hideQcSummary();

    var hero = getQcRoverObservations();
    if (hero != null) {
        hero.style = 'none';
    } 

    var hero = getQcBaseStationsObservations();
    if (hero != null) {
        hero.style = 'none';
    } else {
        console.log("is null");
    }
}

function onQcNaviSummaryConstellationChanges(changes) {
    console.log("YES changes: " + changes.target.value);
}

function onQcNaviSummarySelectionChanges(opts) {
    console.log("CHANGES: "+ opts);
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
}