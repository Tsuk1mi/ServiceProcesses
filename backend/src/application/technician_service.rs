use crate::domain::entities::Technician;
use crate::domain::errors::DomainError;
use crate::ports::outbound::TechnicianRepository;

#[derive(Clone)]
pub struct TechnicianAppService<T>
where
    T: TechnicianRepository,
{
    pub technicians: T,
}

impl<T> TechnicianAppService<T>
where
    T: TechnicianRepository,
{
    pub fn create(
        &self,
        id: String,
        full_name: String,
        skills: Vec<String>,
    ) -> Result<Technician, DomainError> {
        let item = Technician::new(id, full_name, skills)?;
        self.technicians.save(item.clone())?;
        Ok(item)
    }

    pub fn list(&self) -> Result<Vec<Technician>, DomainError> {
        self.technicians.list()
    }
}
