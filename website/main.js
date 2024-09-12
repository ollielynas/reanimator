



function httpGet(theUrl)
{
    var xmlHttp = new XMLHttpRequest();
    xmlHttp.open( "GET", theUrl, false ); // false for synchronous request
    xmlHttp.send( null );
    return xmlHttp.responseText;
}

let data = JSON.parse(httpGet("https://api.github.com/repos/ollielynas/reanimator/releases/latest"));
console.log(data);

let download_link = null;
let i;
for (i in data["assets"]) {
    console.log(i);
    if (
        data["assets"][i]["name"].includes(".msi") &
        !data["assets"][i]["name"].includes(".msi.sha")
    ) {
        download_link = data["assets"][i]["browser_download_url"];
        document.querySelector(".name").innerHTML = data["assets"][i]["name"];
    }
}

window.download_link = download_link;


document.querySelector(".version").innerHTML = "version: " + data["tag_name"];

document.querySelector(".download-button").href = download_link;


const urlParams = new URLSearchParams(window.location.search);
const myParam = urlParams.get('download_latest');


if (!(myParam==null)) {
    window.location.replace(download_link);
    window.location.href = download_link;

}