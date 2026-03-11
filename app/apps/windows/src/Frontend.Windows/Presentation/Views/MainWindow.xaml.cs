using System;
using System.Net.Http;
using System.Windows;
using System.Windows.Controls;
using Frontend.Windows.Infrastructure.Api;
using Frontend.Windows.Infrastructure.Notifications;
using Frontend.Windows.Presentation.ViewModels;

namespace Frontend.Windows.Presentation.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        // В промышленном варианте — DI контейнер. Для каркаса: прямое создание.
        var api = new ApiClient(new HttpClient { BaseAddress = new Uri("https://example.corp/api/") });
        var notifications = new NotificationService();

        DataContext = new MainViewModel(api, notifications);
    }

    private MainViewModel? Vm => DataContext as MainViewModel;

    private void LoginPasswordBox_OnPasswordChanged(object sender, RoutedEventArgs e)
    {
        if (Vm == null) return;
        Vm.LoginPassword = ((PasswordBox)sender).Password;
    }

    private void RegisterPasswordBox_OnPasswordChanged(object sender, RoutedEventArgs e)
    {
        if (Vm == null) return;
        Vm.RegisterPassword = ((PasswordBox)sender).Password;
    }

    private void RegisterPasswordConfirmBox_OnPasswordChanged(object sender, RoutedEventArgs e)
    {
        if (Vm == null) return;
        Vm.RegisterPasswordConfirm = ((PasswordBox)sender).Password;
    }
}
