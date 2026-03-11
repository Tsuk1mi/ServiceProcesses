namespace Frontend.Windows.Infrastructure.Notifications;

/// <summary>
/// A4 Уведомления. В промышленном варианте — интеграция с Toast/Tray/логированием.
/// </summary>
public sealed class NotificationService
{
    public void Info(string message)
    {
        // Заглушка: можно заменить на Toast notifications или event bus.
        Console.WriteLine($"INFO: {message}");
    }

    public void Error(string message)
    {
        Console.WriteLine($"ERROR: {message}");
    }
}


