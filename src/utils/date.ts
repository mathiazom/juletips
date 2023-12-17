import {DateTime, Settings} from "luxon";

export const LocalizedDateTime: typeof DateTime = (() => {
    Settings.defaultLocale = "no";
    Settings.defaultZone = "Europe/Oslo";
    return DateTime;
})();
