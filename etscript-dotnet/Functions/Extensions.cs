namespace Functions;

public static class Extensions
{
    public static string FormatCulture(this string culture)
    {
        return culture.Replace('_', '-');
    }
}
