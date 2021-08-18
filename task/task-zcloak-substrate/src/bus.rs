use lifeline::prelude::*;

use task_management::resource::TaskResource;

lifeline_bus!(pub struct ZcloakTaskBus);

impl Resource<ZcloakTaskBus> for TaskResource {}
