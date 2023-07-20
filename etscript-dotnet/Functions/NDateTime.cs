namespace Functions;

using System.Globalization;
using System.Runtime.InteropServices;

internal enum DateTimeUnit
{
    Year = 1,
    Month,
    Day,
    Hour,
    Minute
}

public static class NDateTime
{
    private static readonly TimeZoneInfo SystemTime;
    private static readonly TimeZoneInfo LocalTime;
    private const string DefaultFormat = "M/d/yyyy h:mm:ss tt";

    static NDateTime()
    {
        const string systemTimeId = "Central Time";
        const string systemTimeName = "(UTC-06:00) Central Time (US & Canada) without daylight saving time";
        SystemTime = TimeZoneInfo.CreateCustomTimeZone(
            systemTimeId,
            new TimeSpan(-6, 0, 0),
            systemTimeName,
            systemTimeId,
            systemTimeId,
            null,
            true
        );

        LocalTime = TimeZoneInfo.Local;

        // long.MinValue --> -9223372036854775808
        // NtpPrimeEpoch --> -2208988800000 (January 1, 1900 00:00:00 UTC)
        // UnixEpoch     -->  0 (January 1, 1970 00:00:00 UTC)
    }

    [UnmanagedCallersOnly(EntryPoint = "n_date_add")]
    public static nint DateAdd(nint dateTimeStringPtr, int addend, int dateTimeUnitId)
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
            var sum = (DateTimeUnit)dateTimeUnitId switch
            {
                DateTimeUnit.Year => dateTime.AddYears(addend),
                DateTimeUnit.Month => dateTime.AddMonths(addend),
                DateTimeUnit.Day => dateTime.AddDays(addend),
                DateTimeUnit.Hour => dateTime.AddHours(addend),
                DateTimeUnit.Minute => dateTime.AddMinutes(addend),
                _ => dateTime
            };

            value = sum.ToString(DefaultFormat);
            data = sum.ToUnixTimeMilliseconds();
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

    [UnmanagedCallersOnly(EntryPoint = "n_date_diff")]
    public static nint DateDiff(nint dateTimeMinuendStringPtr, nint dateTimeSubtrahendStringPtr, int dateTimeUnitId)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var dateTimeMinuendString = Marshal.PtrToStringAnsi(dateTimeMinuendStringPtr);
            if (dateTimeMinuendString == null)
            {
                throw new FormatException("Date-time minuend input string is null.");
            }

            var dateTimeSubtrahendString = Marshal.PtrToStringAnsi(dateTimeSubtrahendStringPtr);
            if (dateTimeSubtrahendString == null)
            {
                throw new FormatException("Date-time subtrahend input string is null.");
            }

            var dateTimeMinuend = DateTimeOffset.Parse(dateTimeMinuendString);
            var dateTimeSubtrahend = DateTimeOffset.Parse(dateTimeSubtrahendString);
            var difference = dateTimeMinuend - dateTimeSubtrahend;

            var dateTimeUnit = (DateTimeUnit)dateTimeUnitId switch
            {
                DateTimeUnit.Year => dateTimeMinuend.Year - dateTimeSubtrahend.Year,
                DateTimeUnit.Month => (dateTimeMinuend.Year - dateTimeSubtrahend.Year) * 12 +
                                      (dateTimeMinuend.Month - dateTimeSubtrahend.Month),
                DateTimeUnit.Day => difference.TotalDays,
                DateTimeUnit.Hour => difference.TotalHours,
                DateTimeUnit.Minute => difference.TotalMinutes,
                _ => 0
            };

            value = Math.Abs(dateTimeUnit).ToString(CultureInfo.InvariantCulture);
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

    [UnmanagedCallersOnly(EntryPoint = "n_date_parse")]
    public static nint DateParse(nint dateTimeStringPtr, int isDateTimeUtc)
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

            var dto = DateTimeOffset.Parse(dateTimeString);
            var dateTime = isDateTimeUtc == 1 ? dto.Subtract(LocalTime.GetUtcOffset(dto)) : dto;

            value = dateTime.ToString(DefaultFormat);
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

    [UnmanagedCallersOnly(EntryPoint = "n_date_part")]
    public static nint DatePart(nint dateTimeStringPtr, int dateTimeUnitId)
    {
        string value;
        const long data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var dateTimeString = Marshal.PtrToStringAnsi(dateTimeStringPtr);
            if (dateTimeString == null)
            {
                throw new FormatException("Date-time input string is null.");
            }

            var dateTime = DateTimeOffset.Parse(dateTimeString);
            var datePart = (DateTimeUnit)dateTimeUnitId switch
            {
                DateTimeUnit.Year => dateTime.ToString("yyyy"),
                DateTimeUnit.Month => dateTime.ToString("MM"),
                DateTimeUnit.Day => dateTime.ToString("dd"),
                DateTimeUnit.Hour => dateTime.ToString("%h"),
                DateTimeUnit.Minute => dateTime.ToString("%m"),
                _ => ""
            };

            value = datePart;
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

    [UnmanagedCallersOnly(EntryPoint = "n_format_date")]
    public static nint FormatDate(nint dateTimeStringPtr, nint dateFormatStringPtr, nint timeFormatStringPtr,
        nint culturePtr)
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

            var dateFormatString = Marshal.PtrToStringAnsi(dateFormatStringPtr);
            if (dateFormatString == null)
            {
                throw new FormatException("Date format input string is null.");
            }

            var timeFormatString = Marshal.PtrToStringAnsi(timeFormatStringPtr);
            if (timeFormatString == null)
            {
                throw new FormatException("Time format input string is null.");
            }

            if (timeFormatString.Length > 0)
            {
                dateFormatString = dateFormatString + " " + timeFormatString;
            }

            var culture = Marshal.PtrToStringAnsi(culturePtr);
            if (culture == null)
            {
                throw new FormatException("Culture input string is null.");
            }

            var formatInfo = CultureInfo.InvariantCulture.DateTimeFormat;
            if (culture.Length > 0)
            {
                formatInfo = CultureInfo.CreateSpecificCulture(culture.FormatCulture()).DateTimeFormat;
            }

            var dateTime = DateTimeOffset.Parse(dateTimeString);

            value = dateTime.ToString(dateFormatString, formatInfo);
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

    [UnmanagedCallersOnly(EntryPoint = "n_local_date_to_system_date")]
    public static nint LocalDateToSystemDate(nint dateTimeStringPtr)
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

            var dateTime = DateTimeOffset.Parse(dateTimeString).ToOffset(SystemTime.BaseUtcOffset);

            value = dateTime.ToString(DefaultFormat);
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

    [UnmanagedCallersOnly(EntryPoint = "n_now")]
    public static nint Now()
    {
        string value;
        var data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var dateTime = DateTimeOffset.Now.ToOffset(SystemTime.BaseUtcOffset);

            value = dateTime.ToString(DefaultFormat);
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

    [UnmanagedCallersOnly(EntryPoint = "n_system_date_to_local_date")]
    public static nint SystemDateToLocalDate(nint dateTimeStringPtr)
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

            var dateTime = DateTimeOffset.Parse(dateTimeString).ToOffset(LocalTime.BaseUtcOffset);

            value = dateTime.ToString(DefaultFormat);
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

    /** Used internally when handling date-time values from a database. */
    [UnmanagedCallersOnly(EntryPoint = "n_system_time_from_unix_time")]
    public static nint SystemTimeFromUnixTime(long unixTime)
    {
        string value;
        var data = long.MinValue;
        var status = (int)Status.Ok;

        try
        {
            var dateTime = DateTimeOffset.FromUnixTimeMilliseconds(unixTime).ToOffset(SystemTime.BaseUtcOffset);

            value = dateTime.ToString(DefaultFormat);
            data = unixTime;
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
