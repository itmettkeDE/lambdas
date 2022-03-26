const AWS_SSO_USER_LIMIT: usize = 50;
const AWS_SSO_GROUP_LIMIT: usize = 50;

pub(crate) type Lookup<T> = std::collections::HashMap<String, T>;

pub(crate) struct SyncOp<'a> {
    scim: &'a crate::aws::Scim<'a>,
    gadmin: &'a crate::google::Admin<'a>,

    aws_group_lookup: Lookup<crate::aws::Group>,
    aws_user_lookup: Lookup<crate::aws::User>,

    google_group_lookup: Lookup<crate::google::Group>,
    google_user_lookup: Lookup<crate::google::User>,
    google_group_assoc: Lookup<std::collections::HashSet<String>>,
}

impl<'a> SyncOp<'a> {
    pub(crate) async fn new(
        event: &'a crate::event::Event,
        scim: &'a crate::aws::Scim<'a>,
        gadmin: &'a crate::google::Admin<'a>,
    ) -> anyhow::Result<SyncOp<'a>> {
        let aws_user_lookup = Self::get_aws_user_lookup(scim).await?;
        let aws_group_lookup = Self::get_aws_group_lookup(scim).await?;
        Self::print_warning(aws_user_lookup.len(), aws_group_lookup.len());

        let google_group_lookup = Self::get_google_group_lookup(event, gadmin).await?;
        let google_user_lookup = Self::get_google_user_lookup(event, gadmin).await?;
        let google_group_assoc =
            Self::get_google_group_assoc_lookup(gadmin, &google_group_lookup).await?;

        Ok(Self {
            scim,
            gadmin,

            aws_group_lookup,
            aws_user_lookup,

            google_group_lookup,
            google_user_lookup,
            google_group_assoc,
        })
    }

    fn print_warning(aws_user_len: usize, aws_group_len: usize) {
        if aws_user_len >= AWS_SSO_USER_LIMIT {
            log::warn!("There are more then 50 users setup in AWS SSO.");
            log::warn!("Currently AWS SSO cannot return more then 50 users.");
            log::warn!(
                "Trying a more difficult method to keep users in sync which will effect performance."
            );
        }
        if aws_group_len >= AWS_SSO_GROUP_LIMIT {
            log::warn!("There are 50 or more groups setup in AWS SSO.");
            log::warn!("Currently AWS SSO cannot return more then 50 groups.");
            log::warn!("Therefore a two-way-sync is not reliable and groups may not be deleted when deleted in google.");
        }
    }

    async fn get_aws_group_lookup(
        scim: &crate::aws::Scim<'_>,
    ) -> anyhow::Result<Lookup<crate::aws::Group>> {
        Ok(scim
            .list_groups()
            .await?
            .into_iter()
            .map(|g| (g.display_name.clone(), g))
            .collect())
    }

    async fn get_aws_user_lookup(
        scim: &crate::aws::Scim<'_>,
    ) -> anyhow::Result<Lookup<crate::aws::User>> {
        Ok(scim
            .list_users()
            .await?
            .into_iter()
            .map(|g| (g.user_name.clone(), g))
            .collect())
    }

    async fn get_google_group_lookup(
        event: &crate::event::Event,
        gadmin: &crate::google::Admin<'_>,
    ) -> anyhow::Result<Lookup<crate::google::Group>> {
        let ignore_groups_regex = event.get_ignore_groups_regexes()?;
        let include_groups_regex = event.get_include_groups_regexes()?;

        Ok(gadmin
            .list_groups(
                event
                    .get_google_api_query_for_groups()
                    .as_ref()
                    .map(AsRef::as_ref),
            )
            .await?
            .into_iter()
            .map(|g| (g.email.clone(), g))
            .filter(|(g, _)| {
                ignore_groups_regex
                    .as_ref()
                    .map_or(true, |r| !r.is_match(g))
            })
            .filter(|(g, _)| {
                include_groups_regex
                    .as_ref()
                    .map_or(true, |r| r.is_match(g))
            })
            .collect::<Lookup<_>>())
    }

    async fn get_google_user_deleted_lookup(
        gadmin: &crate::google::Admin<'_>,
    ) -> anyhow::Result<Lookup<crate::google::User>> {
        Ok(gadmin
            .list_users(None, true)
            .await?
            .into_iter()
            .map(|g| (g.primary_email.clone(), g))
            .collect::<Lookup<_>>())
    }

    async fn get_google_user_lookup(
        event: &crate::event::Event,
        gadmin: &crate::google::Admin<'_>,
    ) -> anyhow::Result<Lookup<crate::google::User>> {
        let ignore_users_regex = event.get_ignore_users_regexes()?;
        let include_users_regex = event.get_include_users_regexes()?;
        Ok(gadmin
            .list_users(
                event
                    .get_google_api_query_for_users()
                    .as_ref()
                    .map(AsRef::as_ref),
                false,
            )
            .await?
            .into_iter()
            .map(|g| (g.primary_email.clone(), g))
            .filter(|(g, _)| ignore_users_regex.as_ref().map_or(true, |r| !r.is_match(g)))
            .filter(|(g, _)| include_users_regex.as_ref().map_or(true, |r| r.is_match(g)))
            .collect::<Lookup<_>>())
    }

    fn modify_google_user_lookup_by_membership(&mut self) {
        let google_user_lookup = &mut self.google_user_lookup;

        let mut users = Lookup::new();
        for members in self.google_group_assoc.values() {
            users.extend(
                members
                    .iter()
                    .filter_map(|member| google_user_lookup.remove_entry(member)),
            );
        }
        self.google_user_lookup = users;
    }

    async fn get_google_group_assoc_lookup(
        gadmin: &crate::google::Admin<'_>,
        google_group_lookup: &Lookup<crate::google::Group>,
    ) -> anyhow::Result<Lookup<std::collections::HashSet<String>>> {
        let mut lookup = Lookup::new();
        for group in google_group_lookup.values() {
            let members = gadmin.list_group_members(&group.id).await?;
            let _ = lookup.insert(group.email.clone(), members);
        }
        Ok(lookup)
    }

    pub(crate) async fn sync_groups(&mut self) -> anyhow::Result<()> {
        self.delete_groups().await?;
        self.add_groups().await?;
        Ok(())
    }

    async fn delete_groups(&mut self) -> anyhow::Result<()> {
        let to_delete = self
            .aws_group_lookup
            .iter()
            .filter(|(id, _)| !self.google_group_lookup.contains_key(*id))
            .filter_map(|(id, u)| {
                Some((id.clone(), u.display_name.clone(), u.id.as_ref()?.clone()))
            })
            .collect::<Vec<_>>();
        for (id, display_name, aws_id) in to_delete {
            log::info!("Deleting group: {}", display_name);
            self.scim.delete_group(&aws_id).await?;
            let _ = self.aws_group_lookup.remove(&id);
        }
        Ok(())
    }

    async fn add_groups(&mut self) -> anyhow::Result<()> {
        let to_remove = self
            .google_group_lookup
            .iter()
            .filter(|(id, _)| self.aws_group_lookup.get(*id).is_none())
            .map(|(_, group)| group)
            .collect::<Vec<_>>();
        for g_group in to_remove {
            log::info!("Creating group: {}", g_group.email);
            let group = Self::create_group(g_group);
            let group = match self.scim.create_group(group).await? {
                Some(g) => g,
                None => {
                    log::info!("Group already exists - fetching instead");
                    self.scim.get_group(&g_group.email).await?
                }
            };
            let _ = self
                .aws_group_lookup
                .insert(group.display_name.clone(), group);
        }
        Ok(())
    }

    fn create_group(group: &crate::google::Group) -> crate::aws::Group {
        crate::aws::Group {
            id: None,
            display_name: group.email.to_owned(),
        }
    }

    pub(crate) async fn sync_users(
        &mut self,
        sync_strategie: crate::event::SyncStrategie,
    ) -> anyhow::Result<()> {
        let require_advanced = self.aws_user_lookup.len() >= AWS_SSO_USER_LIMIT;
        match sync_strategie {
            crate::event::SyncStrategie::AllUsers => {}
            crate::event::SyncStrategie::GroupMembersOnly => {
                self.modify_google_user_lookup_by_membership()
            }
        }
        self.delete_users_simple().await?;
        if require_advanced {
            self.delete_users_advanced().await?;
        }
        self.add_users().await?;
        Ok(())
    }

    async fn delete_users_simple(&mut self) -> anyhow::Result<()> {
        let to_delete = self
            .aws_user_lookup
            .iter()
            .filter(|(id, _)| !self.google_user_lookup.contains_key(*id))
            .filter_map(|(id, u)| Some((id.clone(), u.user_name.clone(), u.id.as_ref()?.clone())))
            .collect::<Vec<_>>();
        for (id, user_name, aws_id) in to_delete {
            log::info!("Deleting user: {}", user_name);
            self.scim.delete_user(&aws_id).await?;
            let _ = self.aws_user_lookup.remove(&id);
        }
        Ok(())
    }

    async fn delete_users_advanced(&mut self) -> anyhow::Result<()> {
        let google_user_active_lookup = Self::get_google_user_deleted_lookup(self.gadmin).await?;
        for (user_id, user) in google_user_active_lookup {
            if self.google_user_lookup.contains_key(&user_id) {
                continue;
            }
            if let Some(user_id) = self
                .scim
                .get_user(&user.primary_email)
                .await?
                .and_then(|u| u.id)
            {
                self.scim.delete_user(&user_id).await?;
            }
        }
        Ok(())
    }

    async fn add_users(&mut self) -> anyhow::Result<()> {
        use anyhow::Context;

        let to_remove = self
            .google_user_lookup
            .iter()
            .filter(|(id, _)| !self.aws_user_lookup.contains_key(*id))
            .map(|(_, user)| user)
            .collect::<Vec<_>>();
        for g_user in to_remove {
            log::info!("Creating user: {}", g_user.primary_email);
            let user = Self::create_user(g_user);
            let user = match self.scim.create_user(user).await? {
                Some(u) => u,
                None => {
                    log::info!("User already exists - fetching instead");
                    self.scim
                        .get_user(&g_user.primary_email)
                        .await?
                        .with_context(|| {
                            format!(
                                "Unable to find user with user_name: {}",
                                g_user.primary_email
                            )
                        })?
                }
            };
            let _ = self.aws_user_lookup.insert(user.user_name.clone(), user);
        }
        Ok(())
    }

    fn create_user(user: &crate::google::User) -> crate::aws::User {
        crate::aws::User {
            id: None,
            external_id: Some(user.id.to_owned()),
            user_name: user.primary_email.to_owned(),
            name: crate::aws::UserName {
                formatted: Some(user.name.full_name.clone()),
                family_name: user.name.family_name.clone(),
                given_name: user.name.given_name.clone(),
            },
            display_name: user.name.full_name.clone(),
            profile_url: user.thumbnail_photo_url.clone(),
            emails: user
                .emails
                .iter()
                .filter(|email| email.primary == Some(true))
                .map(|email| {
                    vec![crate::aws::UserMail {
                        value: email.address.clone(),
                        r#type: email.r#type.clone(),
                        primary: email.primary,
                    }]
                })
                .next(),
            active: !user.suspended.unwrap_or(false),
        }
    }

    pub(crate) async fn sync_associations(&mut self) -> anyhow::Result<()> {
        for (group_id, members) in &self.google_group_assoc {
            let aws_group_id = match self
                .aws_group_lookup
                .get(group_id)
                .and_then(|g| g.id.as_ref())
            {
                Some(v) => v,
                None => continue,
            };
            for (user_id, user) in &self.aws_user_lookup {
                let aws_user_id = match user.id {
                    Some(ref v) => v,
                    None => continue,
                };
                let aws_is_member = self.scim.is_group_member(aws_group_id, aws_user_id).await?;
                let google_is_member = members.contains(user_id);
                if google_is_member && !aws_is_member {
                    log::info!("Adding user {} to group {}.", user_id, group_id);
                    self.scim
                        .add_group_member(aws_group_id, aws_user_id)
                        .await?;
                } else if !google_is_member && aws_is_member {
                    log::info!("Removing user {} from group {}.", user_id, group_id);
                    self.scim
                        .remove_group_member(aws_group_id, aws_user_id)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
