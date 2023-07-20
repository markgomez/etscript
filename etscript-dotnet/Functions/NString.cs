namespace Functions;

using System.Globalization;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;

internal enum DataFormat
{
    Unknown,
    Date,
    Number
}

public static class NString
{
    [UnmanagedCallersOnly(EntryPoint = "n_format")]
    public static nint Format(nint inputStringPtr, nint formatStringPtr, int dataFormatId,
        nint culturePtr)
    {
        string value;
        var data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var inputString = Marshal.PtrToStringAnsi(inputStringPtr);
            if (inputString == null)
            {
                throw new FormatException("Input string is null.");
            }

            var formatString = Marshal.PtrToStringAnsi(formatStringPtr);
            if (formatString == null)
            {
                throw new FormatException("Format input string is null.");
            }

            var culture = Marshal.PtrToStringAnsi(culturePtr);
            if (culture == null)
            {
                throw new FormatException("Culture input string is null.");
            }

            var ci = CultureInfo.InvariantCulture;
            if (culture.Length > 0)
            {
                ci = CultureInfo.CreateSpecificCulture(culture.FormatCulture());
            }

            if ((DataFormat)dataFormatId == DataFormat.Unknown)
            {
                if (DateTime.TryParse(inputString, ci, out _))
                {
                    dataFormatId = (int)DataFormat.Date;
                }
                else
                {
                    dataFormatId = (int)DataFormat.Number;
                }
            }

            if ((DataFormat)dataFormatId == DataFormat.Date)
            {
                var dateTime = DateTimeOffset.Parse(inputString);
                var formatInfo = ci.DateTimeFormat;

                value = dateTime.ToString(formatString, formatInfo);
                data = dateTime.ToUnixTimeMilliseconds();
            }
            else
            {
                var number = double.Parse(inputString);
                var formatInfo = ci.NumberFormat;

                value = number.ToString(formatString, formatInfo);
            }
        }
        catch (Exception e)
        {
            value = e.Message;
            status = (int)Status.Error;
        }

        var result = new NResult(value, data, status);
        var ptr = Marshal.AllocHGlobal(Marshal.SizeOf(result));

        Marshal.StructureToPtr(result, ptr, false);

        return ptr;
    }

    [UnmanagedCallersOnly(EntryPoint = "n_proper_case")]
    public static nint ProperCase(nint inputStringPtr)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var inputString = Marshal.PtrToStringAnsi(inputStringPtr);
            if (inputString == null)
            {
                throw new FormatException("Input string is null.");
            }

            var textInfo = CultureInfo.InvariantCulture.TextInfo;
            value = textInfo.ToTitleCase(inputString.ToLower());
        }
        catch (Exception e)
        {
            value = e.Message;
            status = (int)Status.Error;
        }

        var result = new NResult(value, data, status);
        var ptr = Marshal.AllocHGlobal(Marshal.SizeOf(result));

        Marshal.StructureToPtr(result, ptr, false);

        return ptr;
    }

    [UnmanagedCallersOnly(EntryPoint = "n_regex_match")]
    public static nint RegexMatch(nint inputStringPtr, nint patternStringPtr, nint ordStringPtr,
        nint optionsStringPtr)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var inputString = Marshal.PtrToStringAnsi(inputStringPtr);
            if (inputString == null)
            {
                throw new FormatException("Input string is null.");
            }

            var patternString = Marshal.PtrToStringAnsi(patternStringPtr);
            if (patternString == null)
            {
                throw new FormatException("Pattern input string is null.");
            }

            var ordString = Marshal.PtrToStringAnsi(ordStringPtr);
            if (ordString == null)
            {
                throw new FormatException("Ordinal input string is null.");
            }

            var optionsString = Marshal.PtrToStringAnsi(optionsStringPtr);
            if (optionsString == null)
            {
                throw new FormatException("Options input string is null.");
            }

            void AppendOption(ref RegexOptions options, RegexOptions option)
            {
                if (options == RegexOptions.None)
                {
                    options = option;
                }
                else
                {
                    options |= option;
                }
            }

            var options = RegexOptions.None;
            var optChars = optionsString.Split(',');
            foreach (var optChar in optChars)
            {
                switch (optChar)
                {
                    case "i":
                        AppendOption(ref options, RegexOptions.IgnoreCase);
                        break;

                    case "m":
                        AppendOption(ref options, RegexOptions.Multiline);
                        break;

                    case "n":
                        AppendOption(ref options, RegexOptions.ExplicitCapture);
                        break;

                    case "s":
                        AppendOption(ref options, RegexOptions.Singleline);
                        break;

                    case "x":
                        AppendOption(ref options, RegexOptions.IgnorePatternWhitespace);
                        break;
                }
            }

            var match = Regex.Match(inputString, patternString, options);

            if (ordString.Length > 0)
            {
                if (ordString.All(char.IsDigit))
                {
                    value = match.Success ? match.Groups[int.Parse(ordString)].Value : "";
                }
                else
                {
                    value = match.Success ? match.Groups[ordString].Value : "";
                }
            }
            else
            {
                value = match.Success ? match.Value : "";
            }
        }
        catch (Exception e)
        {
            value = e.Message;
            status = (int)Status.Error;
        }

        var result = new NResult(value, data, status);
        var ptr = Marshal.AllocHGlobal(Marshal.SizeOf(result));

        Marshal.StructureToPtr(result, ptr, false);

        return ptr;
    }

    [UnmanagedCallersOnly(EntryPoint = "n_string_to_date")]
    public static nint StringToDate(nint dateTimeStringPtr)
    {
        string value;
        var data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var dateTimeString = Marshal.PtrToStringAnsi(dateTimeStringPtr);
            if (dateTimeString == null)
            {
                throw new FormatException("Date-time input string is null.");
            }

            var dateTime = DateTimeOffset.Parse(dateTimeString);

            value = dateTime.ToString("M/d/yyyy h:mm:ss tt");
            data = dateTime.ToUnixTimeMilliseconds();
        }
        catch (Exception e)
        {
            value = e.Message;
            status = (int)Status.Error;
        }

        var result = new NResult(value, data, status);
        var ptr = Marshal.AllocHGlobal(Marshal.SizeOf(result));

        Marshal.StructureToPtr(result, ptr, false);

        return ptr;
    }
}
