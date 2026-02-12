// const aprsIconMap = {
//     // Primary Table
//     "/!": "bb", "/\"": "bc", "/#": "bd", "/$": "be", "/%": "bf", "/&": "bg", "/'": "bh",
//     "/(": "bi", "/)": "bj", "/*": "bk", "/+": "bl", "/,": "bm", "/-": "bn", "/.": "bo",
//     "//": "bp", "/0": "p0", "/1": "p1", "/2": "p2", "/3": "p3", "/4": "p4", "/5": "p5",
//     "/6": "p6", "/7": "p7", "/8": "p8", "/9": "p9", "/:": "mr", "/;": "ms", "/<": "mt",
//     "/=": "mu", "/>": "mv", "/?": "mw", "/@": "mx", "/A": "pa", "/B": "pb", "/C": "pc",
//     "/D": "pd", "/E": "pe", "/F": "pf", "/G": "pg", "/H": "ph", "/I": "pi", "/J": "pj",
//     "/K": "pk", "/L": "pl", "/M": "pm", "/N": "pn", "/O": "po", "/P": "pp", "/Q": "pq",
//     "/R": "pr", "/S": "ps", "/T": "pt", "/U": "pu", "/V": "pv", "/W": "pw", "/X": "px",
//     "/Y": "py", "/Z": "pz", "/[": "hs", "/\\": "ht", "/]": "hu", "/^": "hv", "/_": "hw",
//     "/`": "hx", "/a": "la", "/b": "lb", "/c": "lc", "/d": "ld", "/e": "le", "/f": "lf",
//     "/g": "lg", "/h": "lh", "/i": "li", "/j": "lj", "/k": "lk", "/l": "ll", "/m": "lm",
//     "/n": "ln", "/o": "lo", "/p": "lp", "/q": "lq", "/r": "lr", "/s": "ls", "/t": "lt",
//     "/u": "lu", "/v": "lv", "/w": "lw", "/x": "lx", "/y": "ly", "/z": "lz", "/{": "j1",
//     "/|": "j2", "/}": "j3", "/~": "j4",
//     // Alternate Table
//     "\\!": "ob", "\\\"": "oc", "\\#": "od", "\\$": "oe", "\\%": "of", "\\&": "og",
//     "\\'": "oh", "\\(": "oi", "\\)": "oj", "\\*": "ok", "\\+": "ol", "\\,": "om",
//     "\\-": "on", "\\.": "oo", "\\/": "op", "\\0": "a0", "\\1": "a1", "\\2": "a2",
//     "\\3": "a3", "\\4": "a4", "\\5": "a5", "\\6": "a6", "\\7": "a7", "\\8": "a8",
//     "\\9": "a9", "\\:": "nr", "\\;": "ns", "\\<": "nt", "\\=": "nu", "\\>": "nv",
//     "\\?": "nw", "\\@": "nx", "\\A": "aa", "\\B": "ab", "\\C": "ac", "\\D": "ad",
//     "\\E": "ae", "\\F": "af", "\\G": "ag", "\\H": "ah", "\\I": "ai", "\\J": "aj",
//     "\\K": "ak", "\\L": "al", "\\M": "am", "\\N": "an", "\\O": "ao", "\\P": "ap",
//     "\\Q": "aq", "\\R": "ar", "\\S": "as", "\\T": "at", "\\U": "au", "\\V": "av",
//     "\\W": "aw", "\\X": "ax", "\\Y": "ay", "\\Z": "az", "\\[": "ds", "\\\\": "dt",
//     "\\]": "du", "\\^": "dv", "\\_": "dw", "\\`": "dx", "\\a": "sa", "\\b": "sb",
//     "\\c": "sc",  "\\e": "se", "\\f": "sf", "\\g": "sg", "\\h": "sh",
//     "\\i": "si", "\\j": "sj", "\\k": "sk", "\\l": "sl", "\\m": "sm", "\\n": "sn",
//     "\\o": "so", "\\p": "sp", "\\q": "sq", "\\r": "sr", "\\s": "ss", "\\t": "st",
//     "\\u": "su", "\\v": "sv", "\\w": "sw", "\\x": "sx", "\\y": "sy", "\\z": "sz",
//     "\\{": "q1", "\\|": "q2", "\\}": "q3", "\\~": "q4"
// };

const aprsIconMap = {
    "/\"": "bc", "/&": "bg", "\\&": "og", "/W": "<svg  width=\"25.28\" height=\"25.28\" viewBox=\"0 0 25.28 25.28\" xmlns=\"http://www.w3.org/2000/svg\"> <defs id=\"defs1\"> <clipPath clipPathUnits=\"userSpaceOnUse\" id=\"clipPath941\"> <path d=\"M 0,144 H 384 V 0 H 0 Z\" transform=\"translate(-356.44441,-60.224604)\" id=\"path941\" /> </clipPath> </defs> <circle style=\"fill:#0000ff;fill-rule:evenodd;stroke-width:1.19726;fill-opacity:1\" id=\"path1\" cx=\"12.639999\" cy=\"12.639999\" r=\"12.64\" /> <text id=\"text941\" xml:space=\"preserve\" x=\"1.92839\" y=\"17.21427\" style=\"font-size:16px;stroke-width:1.33333\"><tspan style=\"font-variant:normal;font-weight:700;font-stretch:normal;font-size:13.3333px;font-family:Helvetica;writing-mode:lr-tb;fill:#ffffff;fill-opacity:1;fill-rule:nonzero;stroke:none;stroke-width:1.33333\" x=\"1.92839 14.515056\" y=\"17.21427\" id=\"tspan941\">WX</tspan></text> </svg>",
}

async function getSymbol(symbolCode) {
    const svg = aprsIconMap[symbolCode];
    if (svg) {
        return new ol.style.Icon({
            scale: .75,
            src: `data:image/svg+xml;utf8,+${svg}`,
            opacity: 100
        });
    }
    return new ol.style.Icon({
        scale: .75,
        src: `img/aprs-symbols/bc.svg`
    })
}

