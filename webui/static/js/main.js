function switch_themes(name) {
    const basetheme = document.getElementById("basetheme");
    const accounttheme = document.getElementById("accounttheme");

    basetheme.href = `/css/${name}/base.css`;
    accounttheme.href = `/css/${name}/accounts.css`;
}