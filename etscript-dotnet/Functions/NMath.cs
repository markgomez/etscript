namespace Functions;

using System.Globalization;
using System.Runtime.InteropServices;

public static class NMath
{
    [UnmanagedCallersOnly(EntryPoint = "n_format_currency")]
    public static nint FormatCurrency(double number, nint culturePtr, int precision,
        nint symbolPtr)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var culture = Marshal.PtrToStringAnsi(culturePtr);
            if (culture == null)
            {
                throw new FormatException("Culture input string is null.");
            }

            var symbol = Marshal.PtrToStringAnsi(symbolPtr);
            if (symbol == null)
            {
                throw new FormatException("Symbol input string is null.");
            }

            var cultureInfo = CultureInfo.CreateSpecificCulture(culture.FormatCulture());

            if (precision > -1)
            {
                cultureInfo.NumberFormat.CurrencyDecimalDigits = precision;
            }

            if (symbol.Length > 0)
            {
                cultureInfo.NumberFormat.CurrencySymbol = symbol;
            }

            value = number.ToString("C", cultureInfo);
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

    [UnmanagedCallersOnly(EntryPoint = "n_format_number")]
    public static nint FormatNumber(double number, nint formatStringPtr, nint culturePtr)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
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

            var cultureInfo = culture.Length > 0
                ? CultureInfo.CreateSpecificCulture(culture.FormatCulture())
                : CultureInfo.InvariantCulture;

            var formatInfo = cultureInfo.NumberFormat;

            value = number.ToString(formatString, formatInfo);
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
