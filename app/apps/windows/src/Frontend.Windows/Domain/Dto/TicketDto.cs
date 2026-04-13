namespace Frontend.Windows.Domain.Dto;

public class TicketDto
{
    public string Id { get; set; } = null!;
    public string ObjectName { get; set; } = null!;
    public string Status { get; set; } = null!;
    public string Priority { get; set; } = null!;
    public string AssignedTo { get; set; } = null!;
}