function generateRustEnumWithDisplay() {
    const elements = document.querySelectorAll('.glyph.fs1');

    let enumDefinition = "use std::fmt;\n\n#[derive(Debug)]\nenum IconMoon {\n";
    let displayImpl = "\nimpl fmt::Display for IconMoon {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        write!(f, \"{}\", match self {\n";

    elements.forEach(element => {
        const iconSpan = element.querySelector('.mls');
        const input = element.querySelector('fieldset input[type="text"]:first-child');

        if (iconSpan && input) {
            let iconName = iconSpan.textContent.trim().replace('icon-', '');
            iconName = iconName
                .split('-')
                .map((part) => part.charAt(0).toUpperCase() + part.slice(1).toLowerCase())
                .join('');

            enumDefinition += `    ${iconName},\n`;
            displayImpl += `            IconMoon::${iconName} => "\\u{${input.value}}",\n`;
        }
    });

    enumDefinition += "}\n";
    displayImpl += "        })\n    }\n}\n";

    const rustContent = enumDefinition + displayImpl;

    const blob = new Blob([rustContent], {type: 'text/plain'});
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = "icon_moon.rs";

    document.body.appendChild(link);
    link.click();

    document.body.removeChild(link);
}

generateRustEnumWithDisplay();
